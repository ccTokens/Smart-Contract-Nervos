use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::packed::{Byte, CellOutput, ScriptOpt};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use serde_json::Value;
use types::packed::{Tick, Uint128};

use super::super::template_parser::constants::Source;
use super::super::template_parser::{util, CellParser, ScriptParser};
use crate::template_parser::VarParser;

pub struct TickCell {
    pub keyword: String,
}

impl TickCell {
    pub fn new() -> Self {
        Self {
            keyword: String::from("TickCell"),
        }
    }

    fn parse_cell(
        _var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        cell: Value,
        source: Source,
    ) -> Result<(CellOutput, bytes::Bytes), Box<dyn StdError>> {
        // parse capacity of cell
        let capacity = util::parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        // parse cell.lock
        let lock_script = script_parser
            .parse(cell["lock"].clone(), source)
            .map_err(|err| format!("Field `cell.lock` parse failed: {}", err.to_string()))?;

        // parse cell.type
        let type_script = script_parser
            .parse(cell["type"].clone(), source)
            .map_err(|err| format!("Field `cell.type` parse failed: {}", err.to_string()))?;

        let mut args = vec![];
        let version = util::parse_json_u8(
            "Field `cell.type.tmp_data.version`",
            &cell["tmp_data"]["version"],
            Some(0),
        );
        args.extend(version.to_le_bytes().to_vec());

        // parse cell.data Tick
        let tick_data = if cell["tmp_data"].is_null() {
            Tick::default().as_bytes()
        } else {
            let tick_type = util::parse_json_str(
                "Field `cell.tmp_data.Tick.tick_type`",
                &cell["tmp_data"]["Tick"]["tick_type"],
                "mint",
            );
            let tick = if tick_type == "mint" {
                Byte::new(0)
            } else {
                Byte::new(1)
            };

            let token_id = match cell["tmp_data"]["Tick"]["token_id"].as_str() {
                Some(_) => util::parse_json_hex(
                    "Field `cell.tmp_data.Tick.token_id`",
                    &cell["tmp_data"]["Tick"]["token_id"],
                    None,
                ),
                None => {
                    let owner_script = script_parser.parse(
                        cell["tmp_data"]["Tick"]["token_id"].clone(),
                        source,
                    )?
                    .expect("The cell.type.args.owner_script_hash should be a valid Script structure if it is not a hex string.");

                    let owner_script_hash = owner_script.calc_script_hash();
                    owner_script_hash.as_slice().to_vec()
                }
            };
            let value = util::parse_json_u64(
                "Field `cell.tmp_data.Tick.value`",
                &cell["tmp_data"]["Tick"]["value"],
                None,
            ) as u128;
            let merchant = script_parser
                .parse(cell["tmp_data"]["Tick"]["merchant"].clone(), source)
                .map_err(|err| format!("Field `cell.tmp_data.Tick.merchant` parse failed: {}", err.to_string()))?
                .expect("Field `cell.tmp_data.Tick.merchant` is required");
            let merchant = types::packed::Script::from_slice(merchant.as_slice()).unwrap();

            let coin_type = if cell["tmp_data"]["Tick"]["coin_type"].is_null() {
                Vec::<u8>::new()
            } else {
                util::parse_json_hex(
                    "Field `cell.tmp_data.Tick.coin_type`",
                    &cell["tmp_data"]["Tick"]["coin_type"],
                    None,
                )
            };
            let tx_hash = if cell["tmp_data"]["Tick"]["tx_hash"].is_null() {
                Vec::<u8>::new()
            } else {
                util::parse_json_hex(
                    "Field `cell.tmp_data.Tick.tx_hash`",
                    &cell["tmp_data"]["Tick"]["tx_hash"],
                    None,
                )
            };

            let receipt_address = util::parse_json_str(
                "Field `cell.tmp_data.Tick.receipt_addr`",
                &cell["tmp_data"]["Tick"]["receipt_addr"],
                "",
            );

            let tick = Tick::new_builder()
                .tick_type(tick)
                .token_id(token_id.into())
                .value(Uint128::from_slice(value.to_le_bytes().as_slice()).unwrap())
                .merchant(merchant)
                .coin_type(coin_type.into())
                .tx_hash(tx_hash.into())
                .receipt_addr(receipt_address.as_bytes().into())
                .build();
            tick.as_bytes()
        };
        args.extend(tick_data);

        let cell_output = CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script.expect("lock script is required"))
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build();

        let data = args.into();
        Ok((cell_output, data))
    }
}

impl CellParser for TickCell {
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
        // parse outputs[] as a mock cell
        let (cell_output, cell_data) =
            Self::parse_cell(var_parser.clone(), script_parser, data.clone(), Source::Output)
                .map_err(|err| format!("Field `outputs[{}]` parse failed: {}", index, err.to_string()))?;

        Ok((cell_output, cell_data))
    }
}
