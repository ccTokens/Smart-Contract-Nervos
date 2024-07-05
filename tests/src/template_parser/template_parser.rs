use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::{env, fs};

use ckb_testtool::ckb_chain_spec::consensus::TYPE_ID_CODE_HASH;
use ckb_testtool::ckb_jsonrpc_types::{self as rpc_types};
use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::core::{Capacity, Cycle, DepType, ScriptHashType, TransactionBuilder, TransactionView};
use ckb_testtool::ckb_types::packed::*;
use ckb_testtool::ckb_types::prelude::*;
use ckb_testtool::context::{random_type_id_script, Context};
use log::debug;
use rand::{thread_rng, Rng};
use serde_json::Value;

use super::cell_parser::{CellParser, DefaultCell};
use super::script_parser::ScriptParser;
use super::witness_parser::XudtWitness;
use super::{util, VarParser, WitnessParser};

// Environment options
// To specify the binary version of contract, set the environment variable `BINARY_VERSION` to `debug` or `release`.
const ENV_BINARY_VERSION: &str = "BINARY_VERSION";
// To enable the debug print of transaction template, set the environment variable `PRINT_TEMPLATE` to `true`.
const ENV_PRINT_TEMPLATE: &str = "PRINT_TEMPLATE";
// To enable the debug print of transaction, set the environment variable `PRINT_TX` to `true`.
const ENV_PRINT_TX: &str = "PRINT_TX";

pub enum BinaryVersion {
    Debug,
    Release,
}

impl FromStr for BinaryVersion {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(BinaryVersion::Debug),
            "release" => Ok(BinaryVersion::Release),
            _ => Err("Environment variable BINARY_VERSION only support \"debug\" and \"release\"."),
        }
    }
}

pub struct TemplateParser {
    context: RefCell<Context>,
    tx_builder: RefCell<TransactionBuilder>,

    // Parsers
    cell_parsers: HashMap<String, Box<dyn CellParser>>,
    witness_parsers: HashMap<String, Box<dyn WitnessParser>>,
    var_parser: Rc<RefCell<VarParser>>,
    script_parser: RefCell<ScriptParser>,

    // Transaction template
    template: Value,

    // Mock inputs
    mock_inputs: Vec<CellInput>,
}

impl TemplateParser {
    pub fn new(cell_parsers: Vec<Box<dyn CellParser>>, winess_parsers: Vec<Box<dyn WitnessParser>>) -> Self {
        let mut cell_parser_map = HashMap::new();
        for parser in cell_parsers {
            cell_parser_map.insert(parser.get_keyword(), parser);
        }

        let mut witness_parser_map = HashMap::new();
        for parser in winess_parsers {
            witness_parser_map.insert(parser.get_keyword(), parser);
        }

        let default_cell_parser = DefaultCell::new();
        cell_parser_map.insert(default_cell_parser.get_keyword(), Box::new(default_cell_parser));

        let xudt_witness_parser = XudtWitness::new();
        witness_parser_map.insert(xudt_witness_parser.get_keyword(), Box::new(xudt_witness_parser));

        let var_parser = Rc::new(RefCell::new(VarParser::new()));
        let script_parser = RefCell::new(ScriptParser::new(var_parser.clone()));

        TemplateParser {
            context: RefCell::new(Context::default()),
            tx_builder: RefCell::new(TransactionBuilder::default()),
            cell_parsers: cell_parser_map,
            witness_parsers: witness_parser_map,
            var_parser,
            script_parser,
            template: Value::default(),
            mock_inputs: vec![],
        }
    }

    pub fn parse_and_verify(&mut self, tx_template: Value, max_cycles: u64, expected_error: Option<i8>) {
        match env::var(ENV_PRINT_TEMPLATE) {
            Ok(val) => {
                if &val == "true" {
                    debug!(
                        "Transaction template: {}",
                        serde_json::to_string_pretty(&tx_template).unwrap()
                    );
                }
            }
            Err(_) => {}
        };

        match self.parse_value(tx_template) {
            Ok(_) => match self.verify_tx(max_cycles) {
                Ok((cycles, tx_view)) => {
                    debug!(
                        r#"︎↑︎======================================↑︎
    Transaction size: {} bytes,
       Suggested fee: {} shannon(feeRate: 1)
              Cycles: {}
    ========================================"#,
                        tx_view.data().total_size(),
                        tx_view.data().total_size() + 4,
                        cycles
                    );
                }
                Err(err) => match expected_error {
                    Some(expected_error) => {
                        let msg = err.to_string();
                        debug!("Error message(single code): {}", msg);

                        let search = format!("error code {}", expected_error);
                        assert!(
                            msg.contains(search.as_str()),
                            "\n======\nThe test should failed with error code: {}\n======\n",
                            expected_error
                        );
                    }
                    None => {
                        panic!(
                            "\n======\nThe transaction should pass the test, but it failed in script: {}\n======\n",
                            err.to_string()
                        );
                    }
                },
            },
            Err(e) => {
                panic!(
                    "\n======\nParse the template of transaction failed: {}\n======\n",
                    e.to_string()
                );
            }
        }
    }

    #[allow(dead_code)]
    pub fn parse_file(&mut self, tx_filepath: String) -> Result<(), Box<dyn StdError>> {
        let mut raw_json = String::new();
        File::open(tx_filepath)?.read_to_string(&mut raw_json)?;
        self.template = serde_json::from_str(&raw_json)?;
        self.try_parse()
    }

    pub fn parse_value(&mut self, tx_template: Value) -> Result<(), Box<dyn StdError>> {
        self.template = tx_template;
        self.try_parse()
    }

    pub fn try_parse(&mut self) -> Result<(), Box<dyn StdError>> {
        let to_owned = |v: &Vec<Value>| -> Vec<Value> { v.to_owned() };
        debug!("Parsing template ... header_deps");
        if let Some(header_deps) = self.template["header_deps"].as_array().map(to_owned) {
            self.parse_header_deps(header_deps)?
        }
        debug!("Parsing template ... cell_deps");
        if let Some(cell_deps) = self.template["cell_deps"].as_array().map(to_owned) {
            self.parse_cell_deps(cell_deps)?
        }
        debug!("Parsing template ... inputs");
        if let Some(inputs) = self.template["inputs"].as_array().map(to_owned) {
            self.parse_inputs(inputs)?
        }
        debug!("Parsing template ... outputs");
        if let Some(outputs) = self.template["outputs"].as_array().map(to_owned) {
            self.parse_outputs(outputs)?
        }
        debug!("Parsing template ... witnesses");
        if let Some(witnesses) = self.template["witnesses"].as_array().map(to_owned) {
            self.parse_witnesses(witnesses)?
        }

        Ok(())
    }

    pub fn verify_tx(&mut self, max_cycles: u64) -> Result<(Cycle, TransactionView), String> {
        let tx = self.tx_builder.take().build();
        let context = self.context.borrow_mut();

        match env::var(ENV_PRINT_TX) {
            Ok(val) => {
                if &val == "true" {
                    let tx_json = rpc_types::TransactionView::from(tx.clone());
                    debug!("Transaction: {}", serde_json::to_string_pretty(&tx_json).unwrap());
                }
            }
            Err(_) => {}
        };

        match context.verify_tx(&tx, max_cycles) {
            Ok(cycles) => Ok((cycles, tx)),
            Err(err) => Err(format!("Verify script error: {:?}", err.to_string())),
        }
    }

    /// The header_deps should be an array of objects like below:
    ///
    /// ```json
    /// [
    ///     {
    ///         "version": "0x0"
    ///         "number": "0x1c526b",
    ///         "timestamp": "0x179f5e91cb9",
    ///         "epoch": "0x50903410008fb",
    ///         "transactions_root": "0xd5439ebffae718cab0fc837fb7b03a06253c250bcae8a2933ac820580a675560",
    ///         "hash": "0x24e33cfffb2658d32ed61b55ee156ad7288e0980b72416bf3a9ce11ecc2c737a",
    ///         "extra_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
    ///         "nonce": "0xb13000f771a12a98d82d62d2d6dfe382",
    ///         "parent_hash": "0x9fed43a51ae94c039e29602b46d25483c7b6a46cbce48559a3040440e6c12d5d",
    ///         "proposals_hash": "0x1bc5abcadf5b34bcc85b00c4c4afd0d6b01d0c0ca11f8ca34241838d12b9df04",
    ///         "compact_target": "0x1d17319c",
    ///         "dao": "0xece20aa8185bb5368aedb6e329ee24001603723bfefdae0100290badf10d4507",
    ///     },
    ///     ...
    /// ]
    /// ```
    ///
    /// These fields are all optional, and it will be compiled to a RawHeader object in molecule finally.
    fn parse_header_deps(&mut self, header_deps: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        for (i, item) in header_deps.into_iter().enumerate() {
            self.mock_block_header(&format!("header_deps[{}]", i), &item)?;
        }

        Ok(())
    }

    fn parse_cell_deps(&mut self, cell_deps: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_cell_deps = vec![];

        for (i, item) in cell_deps.into_iter().enumerate() {
            let keyword = match item["tmp_type"].as_str() {
                Some(keyword) => keyword,
                _ => match item["out_point"]["tmp_type"].as_str() {
                    Some(keyword) => keyword,
                    _ => "default",
                },
            };

            match keyword {
                "contract" | "deployed_contract" | "shared_lib" | "deployed_shared_lib" => {
                    let tmp_type = item["tmp_type"].as_str().expect("The tmp_type field is required.");
                    let is_deployed = if tmp_type.contains("deployed") { true } else { false };

                    let name = item["tmp_file_name"].as_str().unwrap();
                    let type_args = if item["type_args"].is_null() {
                        //debug!("item = {:#?}", item);
                        None
                    } else {
                        let type_args =
                            util::parse_json_hex(&format!("cell_deps[{}].type_args", i), &item["type_args"], None);
                        Some(bytes::Bytes::from(type_args))
                    };

                    let (_, out_point) = self.mock_contract(name, is_deployed, type_args);
                    let cell_dep = CellDep::new_builder()
                        .out_point(out_point.clone())
                        .dep_type(Byte::new(DepType::Code as u8))
                        .build();

                    mocked_cell_deps.push(cell_dep);
                }
                _ => {
                    match self.cell_parsers.get(keyword) {
                        Some(cell_parser) => {
                            // If we use {{...}} variable in cell_deps, then the contract need to be put in the cell_deps either.
                            // This is because variable is not a real code_hash, but everything needs code_hash here, so the
                            // contract need to be loaded for calculating hash.
                            let (dep_type, cell_output, cell_data) = cell_parser.parse_cell_deps(
                                self.var_parser.clone(),
                                &*self.script_parser.borrow(),
                                item,
                                i,
                            )?;

                            let out_point = self.mock_confirmed_cell(i + 1_000_000, cell_output, cell_data);
                            let cell_dep = CellDep::new_builder()
                                .out_point(out_point.clone())
                                .dep_type(dep_type)
                                .build();

                            mocked_cell_deps.push(cell_dep);
                        }
                        None => {
                            return Err("Unsupported cell_deps type.".into());
                        }
                    }
                }
            }
        }

        let mut builder = self.tx_builder.take();
        builder = builder.set_cell_deps(mocked_cell_deps);
        self.tx_builder.replace(builder);

        Ok(())
    }

    /// The inputs should be an array of objects like below:
    ///
    /// ```json
    /// [
    ///     {
    ///         "previous_output": {
    ///             "capacity": ...,
    ///             "lock": ...,
    ///             "type": ...,
    ///             "tmp_data": ...,""
    ///         },
    ///         "since": "0x...",
    ///     },
    ///     ...
    /// ]
    /// ```
    ///
    /// These fields are all optional, and it will be compiled to a RawHeader object in molecule finally.
    fn parse_inputs(&mut self, inputs: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_inputs = vec![];

        for (i, item) in inputs.into_iter().enumerate() {
            let keyword = match item["tmp_type"].as_str() {
                Some(keyword) => keyword,
                _ => match item["previous_output"]["tmp_type"].as_str() {
                    Some(keyword) => keyword,
                    _ => "default",
                },
            };
            debug!("Parsing inputs keyword = ... {}", keyword);
            match self.cell_parsers.get(keyword) {
                Some(cell_parser) => {
                    let (since, cell_output, cell_data) =
                        cell_parser.parse_inputs(self.var_parser.clone(), &*self.script_parser.borrow(), item, i)?;

                    // debug!("inputs[{}]: {}", i, cell_output);

                    let out_point = self.mock_confirmed_cell(i, cell_output, cell_data);
                    let cell_input = CellInput::new_builder()
                        .previous_output(out_point.clone())
                        .since(since.pack())
                        .build();

                    self.mock_inputs.push(cell_input.clone());
                    mocked_inputs.push(cell_input);
                }
                None => {
                    return Err("Unsupported inputs type.".into());
                }
            }
        }

        let mut builder = self.tx_builder.take();
        builder = builder.set_inputs(mocked_inputs);
        self.tx_builder.replace(builder);

        // eprintln!("Parse self.inputs = {:#?}", self.inputs);
        Ok(())
    }

    fn parse_outputs(&mut self, outputs: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_outputs = Vec::new();
        let mut mocked_outputs_data = Vec::new();

        let first_input = self.mock_inputs.first().expect("The first input should exist.");

        for (i, item) in outputs.into_iter().enumerate() {
            let keyword = match item["tmp_type"].as_str() {
                Some(keyword) => keyword,
                _ => "default",
            };

            self.var_parser
                .borrow_mut()
                .register_var(String::from("type-id"), util::build_type_id_hex(first_input, i as u64));

            match self.cell_parsers.get(keyword) {
                Some(cell_parser) => {
                    let (cell_output, cell_data) =
                        cell_parser.parse_outputs(self.var_parser.clone(), &*self.script_parser.borrow(), item, i)?;

                    mocked_outputs.push(cell_output);
                    mocked_outputs_data.push(cell_data);
                }
                None => {
                    return Err("Unsupported outputs type.".into());
                }
            }
        }

        let mut builder = self.tx_builder.take();
        builder = builder.set_outputs(mocked_outputs);
        builder = builder.set_outputs_data(mocked_outputs_data.into_iter().map(|data| data.pack()).collect());
        self.tx_builder.replace(builder);

        // eprintln!("Parse self.outputs = {:#?}", self.outputs);
        // eprintln!("Parse self.outputs_data = {:#?}", self.outputs_data);
        Ok(())
    }

    fn parse_witnesses(&mut self, witnesses: Vec<Value>) -> Result<(), Box<dyn StdError>> {
        let mut mocked_witnesses = Vec::new();

        for (i, witness) in witnesses.into_iter().enumerate() {
            let data;
            match witness.as_str() {
                Some(_) => {
                    data = bytes::Bytes::from(util::parse_json_hex(
                        format!("witnesses[{}]", i),
                        &witness,
                        Some(vec![]),
                    ));
                }
                None => {
                    let keyword = match witness["tmp_type"].as_str() {
                        Some(keyword) => keyword,
                        _ => "default",
                    };

                    debug!("Parsing witness keyword = ... {}", keyword);
                    match self.witness_parsers.get(keyword) {
                        Some(witness_parser) => {
                            data = witness_parser.parse(
                                self.var_parser.clone(),
                                &*self.script_parser.borrow(),
                                witness,
                                i,
                            )?;
                        }
                        None => {
                            return Err("Unsupported witness type.".into());
                        }
                    }
                }
            }

            //debug!("witness{} = {}", _i, witness.as_str().unwrap());
            //debug!("data{} = {}", _i, hex::encode(data.as_ref()));
            mocked_witnesses.push(data.pack());
        }

        let mut builder = self.tx_builder.take();
        builder = builder.set_witnesses(mocked_witnesses);
        self.tx_builder.replace(builder);

        // eprintln!("Parse self.witnesses = {:#?}", self.witnesses);
        Ok(())
    }

    fn mock_block_header(&self, field_name: &str, header: &Value) -> Result<(), Box<dyn StdError>> {
        let version = util::parse_json_u32(&format!("{}.version", field_name), &header["version"], Some(0));
        let number = if header["number"].is_null() {
            util::parse_json_u64(&format!("{}.height", field_name), &header["height"], Some(0))
        } else {
            util::parse_json_u64(&format!("{}.number", field_name), &header["number"], Some(0))
        };
        let timestamp = util::parse_json_u64(&format!("{}.timestamp", field_name), &header["timestamp"], Some(0));
        let epoch = util::parse_json_u64(&format!("{}.epoch", field_name), &header["epoch"], Some(0));

        let transactions_root_raw = util::parse_json_hex(
            &format!("{}.transactions_root", field_name),
            &header["transactions_root"],
            Some(vec![0u8; 32]),
        );
        let transactions_root = match Byte32::from_slice(&transactions_root_raw) {
            Ok(transactions_root) => transactions_root,
            Err(err) => return Err(format!("Parse transactions_root error: {:?}", err).into()),
        };

        let raw_header = RawHeaderBuilder::default()
            .version(version.pack())
            .number(number.pack())
            .timestamp(timestamp.pack())
            .epoch(epoch.pack())
            .transactions_root(transactions_root)
            .build();
        let header = Header::new_builder().raw(raw_header).nonce(Uint128::default()).build();
        let header_view = header.into_view();

        let mut context = self.context.borrow_mut();
        context.insert_header(header_view);

        Ok(())
    }

    fn mock_confirmed_cell(&self, index: usize, cell: CellOutput, data: bytes::Bytes) -> OutPoint {
        let tx_hash = index_to_byte32(index);
        let out_point = OutPoint::new_builder().index(0u32.pack()).tx_hash(tx_hash).build();

        let mut context = self.context.borrow_mut();
        context.create_cell_with_out_point(out_point.clone(), cell, data);

        out_point
    }

    fn mock_contract(
        &self,
        binary_name: &str,
        is_deployed: bool,
        type_args: Option<bytes::Bytes>,
    ) -> (Byte32, OutPoint) {
        let mut context = self.context.borrow_mut();
        //debug!("binary_name={}, is_deployed={}", binary_name, is_deployed);
        //debug!("type_args: {:?}", hex_string(type_args.clone().unwrap().as_ref()));

        // Create the cell of script
        let file = self.load_binary(binary_name, is_deployed);
        let out_point = self.deploy_cell_with_type_args(&mut context, file, type_args);
        // Get its type ID
        let (cell, _) = context
            .cells
            .get(&out_point)
            .expect("The cell of contract is just created, so it should be exist.");
        let type_script = cell
            .type_()
            .to_opt()
            .expect("The cell of contract should always have a type script.");
        let type_id = type_script.calc_script_hash();
        let type_id_str = util::bytes_to_hex(type_id.as_reader().raw_data());

        // Save the map for later usage
        let mut script_parser = self.script_parser.take();
        script_parser.register_script(binary_name.to_string(), type_id_str);
        self.script_parser.replace(script_parser);

        let type_id_bytes = type_id.as_reader().raw_data();
        debug!("Creating mock type ID:");
        debug!("  name: {}", binary_name,);
        debug!("  type_id: {}", hex::encode(type_id_bytes));
        debug!("  type_script: {}", type_script);

        (type_id, out_point)
    }

    fn load_binary(&self, name: &str, is_deployed: bool) -> bytes::Bytes {
        let current_dir = env::current_dir().unwrap();
        let mut file_path = PathBuf::new();

        if is_deployed {
            file_path.push(current_dir);
            file_path.push("..");
            file_path.push("deployed-scripts");
        } else {
            let binary_version = match env::var(ENV_BINARY_VERSION) {
                Ok(val) => val.parse().expect("Binary version should be one of debug and release."),
                Err(_) => BinaryVersion::Debug,
            };
            let binary_dir = match binary_version {
                BinaryVersion::Debug => "debug",
                BinaryVersion::Release => "release",
            };

            file_path.push(current_dir);
            file_path.push("..");
            file_path.push("build");
            file_path.push(binary_dir);
        }

        file_path.push(name);

        debug!("  Loading binary {} from {} ...", name, file_path.as_path().display());

        fs::read(file_path.as_path())
            .expect(&format!(
                "Can not load binary of {} from path {}.",
                name,
                file_path.display()
            ))
            .into()
    }

    fn deploy_cell_with_type_args(
        &self,
        context: &mut Context,
        data: bytes::Bytes,
        type_args: Option<bytes::Bytes>,
    ) -> OutPoint {
        // WARNING: This requires the fake-* script to have a different data hash; otherwise, the fake-* scripts will conflict.
        let data_hash = CellOutput::calc_data_hash(&data);
        if let Some(out_point) = context.cells_by_data_hash.get(&data_hash) {
            debug!("The cell of data hash {} has been deployed.", data_hash);
            // contract has been deployed
            return out_point.to_owned();
        }

        // mock outpoint
        let mut rng = thread_rng();
        let tx_hash = {
            let mut buf = [0u8; 32];
            rng.fill(&mut buf);
            buf.pack()
        };
        let out_point = OutPoint::new(tx_hash, 0);

        // mock type script and type ID
        let type_id_script = match type_args.clone() {
            Some(type_args) => Script::new_builder()
                .code_hash(TYPE_ID_CODE_HASH.pack())
                .hash_type(ScriptHashType::Type.into())
                .args(type_args.pack())
                .build(),
            None => random_type_id_script(),
        };
        let type_id_hash = type_id_script.calc_script_hash();

        // mock cell
        let cell = {
            let cell = CellOutput::new_builder().type_(Some(type_id_script).pack()).build();
            let occupied_capacity = cell
                .occupied_capacity(Capacity::bytes(data.len()).expect("data occupied capacity"))
                .expect("cell capacity");
            cell.as_builder().capacity(occupied_capacity.pack()).build()
        };

        context.cells.insert(out_point.clone(), (cell, data));
        context.cells_by_data_hash.insert(data_hash, out_point.clone());
        context.cells_by_type_hash.insert(type_id_hash, out_point.clone());
        out_point
    }
}

fn index_to_byte32(index: usize) -> Byte32 {
    let index_bytes = (index as u64).to_be_bytes().to_vec();
    let padding_bytes = [
        vec![0u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        index_bytes,
    ]
    .concat();

    Byte32::from_slice(&padding_bytes).expect("The Byte32::from_slice(&tx_hash_bytes) should always succeed.")
}
