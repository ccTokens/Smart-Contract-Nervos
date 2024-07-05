use std::cell::RefCell;
use std::error::Error as StdError;
use std::rc::Rc;
use std::str::FromStr;

use ckb_testtool::ckb_types::bytes;
use ckb_testtool::ckb_types::packed::{Byte, CellOutput, ScriptOpt};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use serde_json::Value;
use types::constants::GovernanceMemberRole;
use types::packed as cc_types;

use super::super::template_parser::constants::Source;
use super::super::template_parser::{util, CellParser, ScriptParser};
use super::common::parse_version;
use crate::template_parser::VarParser;

pub struct GovernanceMemberCell {
    pub keyword: String,
}

impl GovernanceMemberCell {
    pub fn new() -> Self {
        Self {
            keyword: String::from("GovernanceMemberCell"),
        }
    }

    fn parse_cell(
        var_parser: Rc<RefCell<VarParser>>,
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
        let mut type_script = script_parser
            .parse(cell["type"].clone(), source)
            .map_err(|err| format!("Field `cell.type` parse failed: {}", err.to_string()))?;

        let mut args = vec![];
        if !cell["type"]["args"]["role"].is_null() {
            let role_str = util::parse_json_str("Field `cell.type.args.role`", &cell["type"]["args"]["role"], "");
            let role = GovernanceMemberRole::from_str(role_str)
                .expect("Field `cell.type.args.role` is not a valid GovernanceMemberRole");
            args.extend((role as u8).to_le_bytes().to_vec());
        }
        if !cell["type"]["args"]["cell_id"].is_null() {
            let cell_id = match source {
                Source::CellDep | Source::Input => util::parse_json_hex(
                    "Field `cell.type.args.cell_id`",
                    &cell["type"]["args"]["cell_id"],
                    Some(vec![]),
                ),
                Source::Output => match var_parser.borrow().parse(&cell["type"]["args"]["cell_id"])? {
                    Some(cell_id) => util::hex_to_bytes(&cell_id),
                    None => {
                        util::parse_json_hex("Field `cell.type.args.cell_id`", &cell["type"]["args"]["cell_id"], None)
                    }
                },
            };

            args.extend(cell_id);
        }
        type_script = type_script.map(|mut script| {
            script = script.as_builder().args(args.pack()).build();
            script
        });

        // parse cell.data
        let data;
        if cell["tmp_data"].is_null() {
            data = bytes::Bytes::new()
        } else {
            let version_bytes = parse_version("Field `cell.data.version`", &cell["tmp_data"]["version"])?;

            let mut builder = cc_types::GovernanceMembers::new_builder();

            // parse parent_id
            let parent_id = util::parse_json_hex(
                "Field `cell.tmp_data.parent_id`",
                &cell["tmp_data"]["parent_id"],
                Some(vec![]),
            );
            builder = builder.parent_id(cc_types::Bytes::from_slice(parent_id.pack().as_slice()).unwrap());

            let members = util::parse_json_array("Field `cell.tmp_data.members`", &cell["tmp_data"]["members"]);
            let mut members_mol = cc_types::BytesVec::new_builder();
            let mut members_bytes = vec![];
            for (i, member) in members.iter().enumerate() {
                let pubkey_hash =
                    util::parse_json_hex(format!("Field `cell.tmp_data.members[{}]`", i), &member, Some(vec![]));
                let pubkey_hash_bytes = cc_types::Bytes::from_slice(pubkey_hash.pack().as_slice()).unwrap();

                members_mol = members_mol.push(pubkey_hash_bytes);
                members_bytes.push(pubkey_hash);
            }
            builder = builder.members(members_mol.build());

            let multisig_args = if cell["tmp_data"]["multisig_args"].is_null() {
                cc_types::Bytes::default()
            } else {
                let require_first_n = util::parse_json_u8(
                    "Field `cell.tmp_data.multisig_args.require_first_n`",
                    &cell["tmp_data"]["multisig_args"]["require_first_n"],
                    None,
                );
                let threshold = util::parse_json_u8(
                    "Field `cell.tmp_data.multisig_args.threshold`",
                    &cell["tmp_data"]["multisig_args"]["threshold"],
                    None,
                );
                cc_types::Bytes::from(vec![0, require_first_n, threshold])
            };
            builder = builder.multisig_args(multisig_args);

            // parse lock_args, if not exist, calculate from members
            let lock_args = if cell["tmp_data"]["multisig_args"].is_null() && cell["tmp_data"]["lock_args"].is_null() {
                cc_types::Bytes::default()
            } else {
                let multisig_args = match cell["tmp_data"]["lock_args"].as_str() {
                    // If the lock_args is a string, parse it as hex
                    Some(_) => {
                        util::parse_json_hex("Field `cell.tmp_data.lock_args`", &cell["tmp_data"]["lock_args"], None)
                    }
                    // Otherwise, build it from multisig_args
                    None => {
                        let require_first_n = util::parse_json_u8(
                            "Field `cell.tmp_data.multisig_args.require_first_n`",
                            &cell["tmp_data"]["multisig_args"]["require_first_n"],
                            None,
                        );
                        let threshold = util::parse_json_u8(
                            "Field `cell.tmp_data.multisig_args.threshold`",
                            &cell["tmp_data"]["multisig_args"]["threshold"],
                            None,
                        );

                        util::build_omni_lock_multisig_args(require_first_n, threshold, members_bytes)
                    }
                };

                cc_types::Bytes::from_slice(multisig_args.pack().as_slice()).unwrap()
            };
            builder = builder.lock_args(lock_args);

            let mol_bytes = bytes::Bytes::from(builder.build().as_slice().to_vec());

            data = [version_bytes, mol_bytes].concat().into();
        }

        let cell_output = CellOutput::new_builder()
            .capacity(capacity.pack())
            .lock(lock_script.expect("lock script is required"))
            .type_(ScriptOpt::new_builder().set(type_script).build())
            .build();

        Ok((cell_output, data))
    }
}

impl CellParser for GovernanceMemberCell {
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
