use std::error::Error as StdError;

use ckb_testtool::ckb_types::bytes;
use serde_json::Value;

use super::super::template_parser::util;

pub fn parse_version(field_name: &str, version: &Value) -> Result<bytes::Bytes, Box<dyn StdError>> {
    let version = util::parse_json_u8(field_name, version, None);
    let version_bytes = bytes::Bytes::from(vec![version]);

    Ok(version_bytes)
}
