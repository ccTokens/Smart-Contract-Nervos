use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::packed::{Byte, CellOutput, ScriptOpt};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use log::debug;
use serde_json::Value;
use types::packed::{Byte32, Script, ScriptVecBuilder};
use types::util::blake2b_256;

use super::super::template_parser::constants::Source;
use super::super::template_parser::{util, CellParser, ScriptParser};
use crate::template_parser::VarParser;

pub struct XudtCell {
    pub keyword: String,
}

#[derive(Copy, Clone)]
pub enum XudtFlagsArgs {
    Empty = 0,
    InArgs = 1,
    InWitness = 2,
}
#[derive(Copy, Clone)]
pub enum XudtFlagsOwnerMode {
    InputLock = 0x20,
    OutputType = 0x40,
    InputType = 0x80,
}

impl From<u8> for XudtFlagsArgs {
    fn from(value: u8) -> Self {
        match value {
            0 => XudtFlagsArgs::Empty,
            1 => XudtFlagsArgs::InArgs,
            2 => XudtFlagsArgs::InWitness,
            _ => panic!("Invalid XudtFlags value: {}", value),
        }
    }
}
impl From<u8> for XudtFlagsOwnerMode {
    fn from(value: u8) -> Self {
        match value {
            0x20 => XudtFlagsOwnerMode::InputLock,
            0x40 => XudtFlagsOwnerMode::OutputType,
            0x80 => XudtFlagsOwnerMode::InputType,
            _ => panic!("Invalid XudtFlags value: {}", value),
        }
    }
}
impl XudtFlagsOwnerMode {
    pub fn enable_owner_mode(&self, origin_flag: u8) -> u32 {
        let mut ret = origin_flag as u32;
        match self {
            XudtFlagsOwnerMode::InputLock => {
                ret &= 0xFEFFFFFF;
            }
            XudtFlagsOwnerMode::OutputType => {
                ret |= 0x40000000;
            }
            XudtFlagsOwnerMode::InputType => {
                ret |= 0x80000000;
            }
        }
        ret
    }
}
impl XudtCell {
    pub fn new() -> Self {
        Self {
            keyword: String::from("XudtCell"),
        }
    }

    fn parse_cell(
        _var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        cell: Value,
        source: Source,
    ) -> Result<(CellOutput, bytes::Bytes), Box<dyn StdError>> {
        debug!("parse_cell in XudtCell");
        // parse capacity of cell
        let capacity = util::parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        // parse cell.lock
        let lock_script = script_parser
            .parse(cell["lock"].clone(), source)
            .map_err(|err| format!("Field `cell.lock` parse failed: {}", err.to_string()))?;

        // parse cell.type
        let mut type_script = script_parser
            .parse(cell["type"].clone(), source)
            .map_err(|err| format!("Field `cell.type` parse failed: {}", err.to_string()))?;

        debug!("cell[\"type\"]: {:?}", cell["type"]["args"]["owner_script_hash"]);

        let mut args = vec![];
        match cell["type"]["args"]["owner_script_hash"].as_str() {
            Some(_) => {
                let owner_script_hash = util::parse_json_hex(
                    "cell.type.args.owner_script_hash",
                    &cell["type"]["args"]["owner_script_hash"],
                    None,
                );
                args.extend(&owner_script_hash);
            }
            None => {
                let owner_script = script_parser.parse(
                    cell["type"]["args"]["owner_script_hash"].clone(),
                    source,
                )?
                .expect("The cell.type.args.owner_script_hash should be a valid Script structure if it is not a hex string.");

                let owner_script_hash = owner_script.calc_script_hash();
                args.extend(owner_script_hash.as_slice());
            }
        }

        let xudt_flag = if !cell["type"]["args"]["xudt_args"]["flags"].is_null() {
            let flags = util::parse_json_u8(
                "Field `cell.type.args.xudt_args.flags.args`",
                &cell["type"]["args"]["xudt_args"]["flags"]["args"],
                Some(0),
            );
            if flags > 2 {
                return Err(format!(
                    "Field `cell.type.args.xudt_args.flags` is not a valid XudtFlags: {}",
                    flags
                )
                .into());
            }
            if !cell["type"]["args"]["xudt_args"]["flags"]["owner_mode"].is_null() {
                let owner_mode = util::parse_json_u8(
                    "Field `cell.type.args.xudt_args.flags.owner_mode`",
                    &cell["type"]["args"]["xudt_args"]["flags"]["owner_mode"],
                    Some(0),
                );
                let owner_mode = XudtFlagsOwnerMode::from(owner_mode);
                let flags_final = owner_mode.enable_owner_mode(flags);
                // debug!("xudt_flag is {}", flags_final);

                // let u32flags = flags as u32;
                args.extend(flags_final.to_le_bytes().to_vec());
            } else {
                args.extend((flags as u32).to_le_bytes().to_vec());
            }
            flags.into()
        } else {
            // The flags can be omitted in args
            0.into()
        };

        match xudt_flag {
            XudtFlagsArgs::Empty => {
                // do nothing
            }
            XudtFlagsArgs::InArgs => {
                if !cell["type"]["args"]["xudt_args"]["ScriptVec"].is_null() {
                    let script_vec = cell["type"]["args"]["xudt_args"]["ScriptVec"]
                        .as_array()
                        .expect("Field `cell.type.args.xudt_args.ScriptVec` is not a valid array");

                    let mut script_vec_builder = ScriptVecBuilder::default();
                    for script in script_vec {
                        let mol_script = script_parser
                            .parse(script.clone(), source)
                            .map_err(|err| format!("Field `ScriptVec.Script` parse failed: {}", err.to_string()))?;
                        if mol_script.is_none() {
                            return Err("Field `ScriptVec.Script` is required".to_string().into());
                        }
                        script_vec_builder =
                            script_vec_builder.push(convert_ckb_gen_type_script_to_basic_script(mol_script.unwrap()));
                    }
                    let sv = script_vec_builder.build();
                    match xudt_flag {
                        XudtFlagsArgs::InArgs => {
                            args.extend(sv.as_slice());
                        }
                        XudtFlagsArgs::InWitness => {
                            //todo: not verify this branch
                            let hash = blake2b_256(sv.as_bytes());
                            args.extend(&hash[0..20]); // blake160
                        }
                        _ => {}
                    }
                    //args.extend(sv.as_slice());
                    sv
                } else {
                    ScriptVecBuilder::default().build()
                };
            }
            XudtFlagsArgs::InWitness => unimplemented!(),
        }

        type_script = type_script.map(|mut script| {
            script = script.as_builder().args(args.pack()).build();
            script
        });

        // parse cell.data
        let data = if cell["tmp_data"].is_null() {
            bytes::Bytes::new()
        } else {
            let sudt_amount =
                util::parse_json_u64("Field `cell.tmp_data.amount`", &cell["tmp_data"]["amount"], None) as u128;
            let sudt_amount_bytes = sudt_amount.to_le_bytes().to_vec();
            bytes::Bytes::from(sudt_amount_bytes)
        };

        let cell_output = CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script.expect("lock script is required"))
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build();

        Ok((cell_output, data))
    }
}

impl CellParser for XudtCell {
    fn get_keyword(&self) -> String {
        self.keyword.clone()
    }

    fn parse_cell_deps(
        &self,
        var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        data: Value,
        index: usize,
    ) -> Result<(Byte, CellOutput, bytes::Bytes), Box<dyn StdError>> {
        debug!("parse_cell_deps in XudtCell");

        // parse cell_deps[].out_point as a mock cell
        let (cell_output, cell_data) = Self::parse_cell(
            var_parser.clone(),
            script_parser,
            data["out_point"].clone(),
            Source::CellDep,
        )
        .map_err(|err| {
            format!(
                "Field `cell_deps[{}].out_point` parse failed: {}",
                index,
                err.to_string()
            )
        })?;

        // parse cell_deps[].dep_type
        let dep_type = util::parse_json_u8(&format!("cell_deps[{}].dep_type", index), &data["dep_type"], Some(0));
        let dep_type = Byte::new(dep_type);

        Ok((dep_type, cell_output, cell_data))
    }

    fn parse_inputs(
        &self,
        var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        data: Value,
        index: usize,
    ) -> Result<(u64, CellOutput, bytes::Bytes), Box<dyn StdError>> {
        debug!("parse_inputs in XudtCell");

        // parse inputs[].previous_output as a mock cell
        let (cell_output, cell_data) = Self::parse_cell(
            var_parser.clone(),
            script_parser,
            data["previous_output"].clone(),
            Source::Input,
        )
        .map_err(|err| {
            format!(
                "Field `inputs[{}].previous_output` parse failed: {}",
                index,
                err.to_string()
            )
        })?;

        // parse inputs[].since
        let since = util::parse_json_u64(&format!("inputs[{}].since", index), &data["since"], Some(0));

        Ok((since, cell_output, cell_data))
    }

    fn parse_outputs(
        &self,
        var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        data: Value,
        index: usize,
    ) -> Result<(CellOutput, bytes::Bytes), Box<dyn StdError>> {
        debug!("parse_outputs in XudtCell");

        // parse outputs[] as a mock cell
        let (cell_output, cell_data) =
            Self::parse_cell(var_parser.clone(), script_parser, data.clone(), Source::Output)
                .map_err(|err| format!("Field `outputs[{}]` parse failed: {}", index, err.to_string()))?;

        Ok((cell_output, cell_data))
    }
}

fn convert_ckb_gen_type_script_to_basic_script(script: ckb_testtool::ckb_types::packed::Script) -> Script {
    let code_hash = script.code_hash().raw_data().to_vec();
    let hash_type = script.hash_type().into();
    let args = script.args().raw_data().to_vec();
    let code_hash_bytes32 = Byte32::new_unchecked(code_hash.into());
    Script::new_builder()
        .code_hash(code_hash_bytes32)
        .hash_type(hash_type)
        .args(args.into())
        .build()
}
