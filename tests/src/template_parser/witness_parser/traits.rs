use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::bytes;
use serde_json::Value;

use super::super::script_parser::ScriptParser;
use super::super::var_parser::VarParser;

pub trait WitnessParser {
    fn get_keyword(&self) -> String;

    fn parse(
        &self,
        var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        input: Value,
        index: usize,
    ) -> Result<bytes::Bytes, Box<dyn StdError>>;
}
