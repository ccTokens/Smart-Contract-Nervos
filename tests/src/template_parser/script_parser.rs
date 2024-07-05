use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::core::ScriptHashType;
use ckb_testtool::ckb_types::packed::{Byte32, Script};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use log::debug;
use serde_json::Value;
use types::packed as cc_types;

use super::constants::Source;
use super::{util, VarParser};

#[derive(Clone, Debug, Default)]
pub struct ScriptParser {
    var_parser: Rc<RefCell<VarParser>>,
}

impl ScriptParser {
    pub fn new(var_parser: Rc<RefCell<VarParser>>) -> ScriptParser {
        ScriptParser { var_parser }
    }

    pub fn register_script(&mut self, binary_name: String, type_id: String) {
        self.var_parser.borrow_mut().register_var(binary_name, type_id);
    }

    pub fn parse(&self, script_val: Value, _source: Source) -> Result<Option<Script>, Box<dyn StdError>> {
        if script_val.is_null() {
            return Ok(None);
        } else if script_val["code_hash"].as_str().is_none() {
            return Err("The code_hash field is required.".into());
        }

        let var_parser = self.var_parser.borrow();

        let code_hash_hex = match var_parser.parse(&script_val["code_hash"])? {
            // If the code_hash field is a variable like {{xxx}}, then parse it to real type ID of contract.
            Some(code_hash) => {
                // Print the replacement for easier debugging.
                debug!(
                    "Replace code_hash {} with {}",
                    script_val["code_hash"].as_str().unwrap(),
                    code_hash
                );

                code_hash
            }
            // Otherwise, parse the code_hash as is.
            None => script_val["code_hash"]
                .as_str()
                .expect("The code_hash field is required.")
                .to_string(),
        };

        let args_hex = match var_parser.parse(&script_val["args"])? {
            Some(args) => args,
            None => script_val["args"].as_str().unwrap_or("").to_string(),
        };

        let code_hash = Byte32::from_slice(cc_types::Byte32::from(code_hash_hex.as_str()).as_slice())
            .expect("The Byte32 in different libraries should be compatible.");
        let args = bytes::Bytes::from(util::hex_to_bytes(&args_hex)).pack();
        let hash_type = match script_val["hash_type"].as_str() {
            Some("data") => ScriptHashType::Data,
            _ => ScriptHashType::Type,
        };

        let script = Some(
            Script::new_builder()
                .code_hash(code_hash)
                .hash_type(hash_type.into())
                .args(args)
                .build(),
        );

        Ok(script)
    }
}
