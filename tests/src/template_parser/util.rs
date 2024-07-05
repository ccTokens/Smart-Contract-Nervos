use std::str;

use ckb_testtool::ckb_hash::{blake2b_256, new_blake2b};
use ckb_testtool::ckb_types::packed;
use ckb_testtool::ckb_types::prelude::Entity;
use serde_json::Value;

use super::constants::{OMNI_FLAG_MULTISIG, OMNI_FLAG_NO_MODE};

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

pub fn build_type_id(input: &packed::CellInput, output_index: u64) -> [u8; 32] {
    let mut blake2b = new_blake2b();
    blake2b.update(input.as_slice());
    blake2b.update(&output_index.to_le_bytes());
    let mut ret = [0; 32];
    blake2b.finalize(&mut ret);

    ret
}

pub fn build_type_id_hex(input: &packed::CellInput, output_index: u64) -> String {
    let type_id = build_type_id(input, output_index);
    bytes_to_hex(&type_id)
}

pub fn build_multisig_args(require_first_n: u8, threshold: u8, pubkey_hashes: Vec<Vec<u8>>) -> Vec<u8> {
    let mut bytes = vec![0, require_first_n, threshold, pubkey_hashes.len() as u8];

    for pubkey_hash in pubkey_hashes.iter() {
        bytes.extend(pubkey_hash);
    }

    // debug!("bytes: {:?}", bytes);

    let hash = blake2b_256(bytes);
    (&hash[..20]).to_vec()
}

pub fn build_omni_lock_multisig_args(require_first_n: u8, threshold: u8, pubkey_hashes: Vec<Vec<u8>>) -> Vec<u8> {
    let lock_args = build_multisig_args(require_first_n, threshold, pubkey_hashes);

    let mut ret = vec![OMNI_FLAG_MULTISIG];
    ret.extend(lock_args);
    ret.extend(&[OMNI_FLAG_NO_MODE]);

    ret
}

#[allow(dead_code)]
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

/// Parse u64 in JSON
///
/// Support both **number** and **string** format.
pub fn parse_json_u64(field_name: &str, field: &Value, default: Option<u64>) -> u64 {
    if let Some(val) = field.as_u64() {
        val
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u64 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse u32 in JSON
///
/// Support both **number** and **string** format.
pub fn parse_json_u32(field_name: &str, field: &Value, default: Option<u32>) -> u32 {
    if let Some(val) = field.as_u64() {
        val as u32
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u32 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse u8 in JSON
pub fn parse_json_u8(field_name: &str, field: &Value, default: Option<u8>) -> u8 {
    if let Some(val) = field.as_u64() {
        if val > u8::MAX as u64 {
            panic!("{} should be u8", field_name)
        } else {
            val as u8
        }
    } else if let Some(val) = field.as_str() {
        val.replace("_", "")
            .parse()
            .expect(&format!("{} should be u8 in string", field_name))
    } else {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name);
        }
    }
}

/// Parse hex string in JSON, if it is not exist return the default value.
pub fn parse_json_hex(field_name: impl AsRef<str>, field: &Value, default: Option<Vec<u8>>) -> Vec<u8> {
    if field.is_null() {
        if let Some(val) = default {
            return val;
        } else {
            panic!("{} is missing", field_name.as_ref());
        }
    } else {
        let mut hex = field.as_str().expect(&format!("{} is missing", field_name.as_ref()));
        hex = hex.trim_start_matches("0x");

        if hex == "" {
            Vec::new()
        } else {
            hex::decode(hex).expect(&format!("{} is should be hex string", field_name.as_ref()))
        }
    }
}

/// Parse string in JSON
///
/// All string will be treated as utf8 encoding.
pub fn parse_json_str<'a>(field_name: &str, field: &'a Value, default: &'a str) -> &'a str {
    if field.is_null() {
        default
    } else {
        field.as_str().expect(&format!("{} is missing", field_name))
    }
}

/// Parse array in JSON
pub fn parse_json_array<'a>(field_name: &str, field: &'a Value) -> &'a [Value] {
    field
        .as_array()
        .map(|v| v.as_slice())
        .expect(&format!("{} is missing", field_name))
}
