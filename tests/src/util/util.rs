#![allow(dead_code)]

use std::io::Write;
use std::str;

use ckb_testtool::ckb_types::core::ScriptHashType;
use ckb_testtool::ckb_types::packed::{BytesOpt, WitnessArgs};
use ckb_testtool::ckb_types::prelude::{Builder, Pack};
use serde_json::{json, Value};
use types::constants::{ConfigKey, Source};
use types::packed::{Byte, Byte32, Bytes, Script};
use types::prelude::Entity;

use crate::custom_parser;
use crate::template_parser::{util as parser_util, CellParser, TemplateParser};
use crate::util::constants::{
    ALWAYS_SUCCESS_TYPE_ARGS, ALWAYS_SUCCESS_TYPE_ID, CUSTODIAN_LOCK_ARGS_1, CUSTODIAN_LOCK_ARGS_2,
    CUSTODIAN_LOCK_ARGS_3, CUSTODIAN_LOCK_ARGS_4, CUSTODIAN_LOCK_ARGS_5, FAKE_OMNI_LOCK_TYPE_ARGS,
    FAKE_OMNI_LOCK_TYPE_ID, GOVERNANCE_MEMBER_CELL_TYPE_ARGS, GOVERNANCE_MEMBER_CELL_TYPE_ID, TICK_CELL_TYPE_ARGS,
    TICK_CELL_TYPE_ID, XUDT_OWNER_TYPE_ARGS, XUDT_OWNER_TYPE_ID, XUDT_RCE_TYPE_ARGS, XUDT_RCE_TYPE_ID,
};

pub fn hex_to_bytes(input: &str) -> Vec<u8> {
    let hex = input.trim_start_matches("0x");
    if hex == "" {
        Vec::new()
    } else {
        hex::decode(hex).expect("Expect input to valid hex")
    }
}

pub fn bytes_to_hex(input: &[u8]) -> String {
    if input.is_empty() {
        String::from("0x")
    } else {
        String::from("0x") + &hex::encode(input)
    }
}

pub fn merge_json(target: &mut Value, source: Value) {
    if source.is_null() {
        return;
    }

    match (target, source) {
        (a @ &mut Value::Object(_), Value::Object(b)) => {
            let a = a.as_object_mut().unwrap();
            for (k, v) in b {
                merge_json(a.entry(k).or_insert(Value::Null), v);
            }
        }
        (a @ &mut Value::Array(_), Value::Array(b)) => {
            let a = a.as_array_mut().unwrap();
            for v in b {
                a.push(v);
            }
        }
        (a, b) => *a = b,
    }
}

pub fn bytes_to_bytes_opt(input: Vec<u8>) -> BytesOpt {
    BytesOpt::new_builder().set(Some(input.pack())).build()
}

pub fn init_template_parser() -> TemplateParser {
    let _ = env_logger::builder()
        .format(|buf, record| writeln!(buf, "[TemplateParser] {}", record.args()))
        .is_test(true)
        .try_init();

    let cell_parsers: Vec<Box<dyn CellParser>> = vec![
        Box::new(custom_parser::GovernanceMemberCell::new()),
        Box::new(custom_parser::ConfigCell::new()),
        Box::new(custom_parser::XudtCell::new()),
        Box::new(custom_parser::TickCell::new()),
    ];

    TemplateParser::new(cell_parsers, vec![])
}

pub fn gen_action(version: u8, action: &str) -> String {
    let mut action_bytes = vec![version];
    action_bytes.extend_from_slice(action.as_bytes());

    bytes_to_hex(action_bytes.as_slice())
}

pub fn gen_tick_cell_witness_args(coin_type: u32, tx_hash: &str, source: Source) -> String {
    // The contract will ignore the specific value, so it could be LE or BE.
    let mut data = vec![];

    data.extend(&4u32.to_le_bytes());
    data.extend(&coin_type.to_le_bytes());

    let tx_hash = hex_to_bytes(tx_hash);

    data.extend(&32u32.to_le_bytes());
    data.extend(&tx_hash);

    let mut builder = WitnessArgs::new_builder();
    if source == Source::Input {
        builder = builder.input_type(bytes_to_bytes_opt(data));
    } else {
        builder = builder.output_type(bytes_to_bytes_opt(data));
    }
    let witness_args = builder.build();

    bytes_to_hex(witness_args.as_slice())
}

pub fn gen_configs() -> Value {
    json!([
        [ConfigKey::SystemStatus as u32, "0x01"],
        [
            ConfigKey::GovernanceMemberCellTypeId as u32,
            GOVERNANCE_MEMBER_CELL_TYPE_ID
        ],
        [
            ConfigKey::GovernanceMemberCellTypeArgs as u32,
            GOVERNANCE_MEMBER_CELL_TYPE_ARGS
        ],
        [ConfigKey::TickCellTypeId as u32, TICK_CELL_TYPE_ID],
        [ConfigKey::TickCellTypeArgs as u32, TICK_CELL_TYPE_ARGS],
        [ConfigKey::XudtInfoCellTypeId as u32, TICK_CELL_TYPE_ID],
        [ConfigKey::XudtInfoCellTypeArgs as u32, TICK_CELL_TYPE_ID],
        [ConfigKey::XudtCellTypeId as u32, XUDT_RCE_TYPE_ID],
        [ConfigKey::XudtCellTypeArgs as u32, XUDT_RCE_TYPE_ARGS],
        [ConfigKey::AlwaysSuccessTypeId as u32, ALWAYS_SUCCESS_TYPE_ID],
        [ConfigKey::AlwaysSuccessTypeArgs as u32, ALWAYS_SUCCESS_TYPE_ARGS],
        [ConfigKey::OmniLockTypeId as u32, FAKE_OMNI_LOCK_TYPE_ID],
        [ConfigKey::OmniLockTypeArgs as u32, FAKE_OMNI_LOCK_TYPE_ARGS],
        [ConfigKey::XudtOwnerTypeId as u32, XUDT_OWNER_TYPE_ID],
        [ConfigKey::XudtOwnerTypeArgs as u32, XUDT_OWNER_TYPE_ARGS],
        // [ConfigKey::XudtOwnerTypeId as u32, FAKE_XUDT_OWNER_TYPE_ID],
        // [ConfigKey::XudtOwnerTypeArgs as u32, FAKE_XUDT_OWNER_TYPE_ARGS],
    ])
}

pub fn gen_custodian_lock_args() -> String {
    let custodian_lock_args = parser_util::build_omni_lock_multisig_args(
        0,
        3,
        vec![
            hex_to_bytes(CUSTODIAN_LOCK_ARGS_1),
            hex_to_bytes(CUSTODIAN_LOCK_ARGS_2),
            hex_to_bytes(CUSTODIAN_LOCK_ARGS_3),
            hex_to_bytes(CUSTODIAN_LOCK_ARGS_4),
            hex_to_bytes(CUSTODIAN_LOCK_ARGS_5),
        ],
    );

    bytes_to_hex(&custodian_lock_args)
}

pub fn gen_merchant_script(args: &str) -> String {
    let args = hex_to_bytes(args);
    let type_id = Byte32::from_slice(&hex_to_bytes(FAKE_OMNI_LOCK_TYPE_ID))
        .expect("The FAKE_OMNI_LOCK_TYPE_ID should be 32 bytes constant.");

    let script = Script::new_builder()
        .code_hash(type_id)
        .hash_type(Byte::new(ScriptHashType::Type as u8))
        .args(Bytes::from(args))
        .build();

    bytes_to_hex(script.as_slice())
}
