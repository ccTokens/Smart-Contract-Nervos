use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;

use ckb_testtool::ckb_types::prelude::{Builder, Entity};
use ckb_testtool::ckb_types::{bytes, packed as ckb_packed};
use serde_json::Value;
use types::packed::{BytesOpt, Script, ScriptOpt, XudtWitnessInput};
use var_parser::VarParser;

use super::super::constants::Source;
use super::super::script_parser::ScriptParser;
use super::super::util;
use super::WitnessParser;
use crate::template_parser::var_parser;

pub struct XudtWitness {
    pub keyword: String,
}

impl XudtWitness {
    pub fn new() -> Self {
        Self {
            keyword: String::from("xudt"),
        }
    }
}

impl WitnessParser for XudtWitness {
    fn get_keyword(&self) -> String {
        self.keyword.clone()
    }

    fn parse(
        &self,
        _var_parser: Rc<RefCell<VarParser>>,
        script_parser: &ScriptParser,
        input: Value,
        index: usize,
    ) -> Result<bytes::Bytes, Box<dyn StdError>> {
        // Parse WitnessArgs.lock
        let lock = util::parse_json_hex(format!("witnesses[{}].lock", index), &input["lock"], Some(vec![]));
        let lock_mol = BytesOpt::from(Some(lock));

        // Build WitnessArgs
        let mut witness_args_builder = ckb_packed::WitnessArgs::new_builder();
        witness_args_builder = witness_args_builder.lock(ckb_packed::BytesOpt::new_unchecked(lock_mol.as_bytes()));

        // Parse WitnessArgs.input_type
        for field in ["input_type", "output_type"] {
            let owner_script = script_parser.parse(input[field]["owner_script"].clone(), Source::Input)?;
            let owner_signature = util::parse_json_hex(
                format!("witnesses[{}].{}.owner_signature", index, field),
                &input[field]["owner_signature"],
                Some(vec![]),
            );

            let mut builder = XudtWitnessInput::new_builder();
            builder = builder.owner_script(ScriptOpt::from(
                owner_script.map(|s| Script::new_unchecked(s.as_bytes())),
            ));
            builder = builder.owner_signature(BytesOpt::from(Some(owner_signature)));
            let xudt_witness_input = builder.build();
            let xudt_witness_input_bytes = xudt_witness_input.as_slice().to_vec();
            let bytes_opt_mol = BytesOpt::from(Some(xudt_witness_input_bytes));

            if field == "input_type" {
                witness_args_builder =
                    witness_args_builder.input_type(ckb_packed::BytesOpt::new_unchecked(bytes_opt_mol.as_bytes()));
            } else {
                witness_args_builder =
                    witness_args_builder.output_type(ckb_packed::BytesOpt::new_unchecked(bytes_opt_mol.as_bytes()));
            }
        }

        let witness_args = witness_args_builder.build();
        Ok(bytes::Bytes::from(witness_args.as_slice().to_vec()))
    }
}
