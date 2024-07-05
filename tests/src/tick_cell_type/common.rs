use serde_json::{json, Value};

use crate::util::constants::TYPE_ID_ARGS;

pub fn gen_xudt_token_id() -> Value {
    json!({
        "code_hash": "{{xudt_owner.so}}",
        "args": TYPE_ID_ARGS
    })
}

pub fn gen_xudt_args() -> Value {
    json!({
        "owner_script_hash": {
            "code_hash": "{{xudt_owner.so}}",
            "args": TYPE_ID_ARGS
        }
    })
}

pub fn gen_xudt_witness() -> Value {
    json!({
        "owner_script": {
            "code_hash": "{{xudt_owner.so}}",
            "args": TYPE_ID_ARGS
        },
    })
}
