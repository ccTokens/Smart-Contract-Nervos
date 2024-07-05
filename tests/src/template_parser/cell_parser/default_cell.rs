use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::packed::{Byte, CellOutput, ScriptOpt};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use serde_json::Value;
use var_parser::VarParser;

use super::super::constants::Source;
use super::super::script_parser::ScriptParser;
use super::super::util;
use super::traits::CellParser;
use crate::template_parser::var_parser;

pub struct DefaultCell {
    pub keyword: String,
}

impl DefaultCell {
    pub fn new() -> Self {
        Self {
            keyword: String::from("default"),
        }
    }

    fn parse_cell(
        &self,
        script_parser: &ScriptParser,
        cell: Value,
        source: Source,
    ) -> Result<(CellOutput, bytes::Bytes), Box<dyn StdError>> {
        // parse capacity of cell
        let capacity = util::parse_json_u64("cell.capacity", &cell["capacity"], Some(0));

        // parse lock script and type script of cell
        let lock_script = script_parser
            .parse(cell["lock"].clone(), source)
            .map_err(|err| format!("Field `cell.lock` parse failed: {}", err.to_string()))?;
        let type_script = script_parser
            .parse(cell["type"].clone(), source)
            .map_err(|err| format!("Field `cell.type` parse failed: {}", err.to_string()))?;

        // parse data of cell
        let data;
        if let Some(hex) = cell["tmp_data"].as_str() {
            data = bytes::Bytes::from(util::hex_to_bytes(hex));
        } else {
            data = bytes::Bytes::new();
        }

        let cell_output = CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script.expect("lock script is required"))
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build();

        Ok((cell_output, data))
    }
}

impl CellParser for DefaultCell {
    fn get_keyword(&self) -> String {
        self.keyword.clone()
    }

    fn parse_cell_deps(
        &self,
        _var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        data: Value,
        index: usize,
    ) -> Result<(Byte, CellOutput, bytes::Bytes), Box<dyn StdError>> {
        // parse cell_deps[].out_point as a mock cell
        let (cell_output, cell_data) = self
            .parse_cell(script_parser, data["out_point"].clone(), Source::CellDep)
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
        _var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        data: Value,
        index: usize,
    ) -> Result<(u64, CellOutput, bytes::Bytes), Box<dyn StdError>> {
        // parse inputs[].previous_output as a mock cell
        let (cell_output, cell_data) = self
            .parse_cell(script_parser, data["previous_output"].clone(), Source::Input)
            .map_err(|err| {
                format!(
                    "Field `inputs[{}].previous_output` parse failed: {}",
                    index,
                    err.to_string()
                )
            })?;

        // parse inputs[].since
        let since = util::parse_json_u64(&format!("inputs[{}].since", index), &data["since"], Some(0));

        // TODO Support mock block header in the inputs if needed
        // if !item["previous_output"]["tmp_header"].is_null() {
        //     let header = self.mock_block_header(
        //         &format!("inputs[{}]", i),
        //         &item["previous_output"]["tmp_header"],
        //     )?;
        // }

        Ok((since, cell_output, cell_data))
    }

    fn parse_outputs(
        &self,
        _var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        data: Value,
        index: usize,
    ) -> Result<(CellOutput, bytes::Bytes), Box<dyn StdError>> {
        // parse outputs[] as a mock cell
        let (cell_output, cell_data) = self
            .parse_cell(script_parser, data.clone(), Source::Output)
            .map_err(|err| format!("Field `outputs[{}]` parse failed: {}", index, err.to_string()))?;

        Ok((cell_output, cell_data))
    }
}
