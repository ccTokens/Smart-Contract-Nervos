use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::packed::{Byte, CellOutput, ScriptOpt};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use serde_json::Value;
use types::packed as cc_types;

use super::super::template_parser::constants::Source;
use super::super::template_parser::{util, CellParser, ScriptParser};
use super::common::parse_version;
use crate::template_parser::VarParser;

pub struct ConfigCell {
    pub keyword: String,
}

impl ConfigCell {
    pub fn new() -> Self {
        Self {
            keyword: String::from("ConfigCell"),
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

        // parse cell.data
        let data = if cell["tmp_data"].is_null() {
            bytes::Bytes::new()
        } else {
            let version_bytes = parse_version("Field `cell.data.version`", &cell["tmp_data"]["version"])?;

            let configs = util::parse_json_array("Field `cell.tmp_data.configs`", &cell["tmp_data"]["configs"]);
            let mut builder = cc_types::BytesVec::new_builder();
            for (i, config) in configs.iter().enumerate() {
                let key = util::parse_json_u32(&format!("Field `cell.tmp_data.config[{}][0]`", i), &config[0], None)
                    .to_le_bytes()
                    .to_vec();
                let value = util::parse_json_hex(&format!("Field `cell.tmp_data.config[{}][1]`", i), &config[1], None);
                let config_bytes = bytes::Bytes::from([key, value].concat());
                builder = builder.push(cc_types::Bytes::from(config_bytes.pack().as_reader().raw_data()));
            }
            let mol_bytes = bytes::Bytes::from(builder.build().as_slice().to_vec());

            [version_bytes, mol_bytes].concat().into()
        };

        let cell_output = CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script.expect("lock script is required"))
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build();

        Ok((cell_output, data))
    }
}

impl CellParser for ConfigCell {
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
