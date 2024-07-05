use serde_json::json;

use crate::util;
use crate::util::constants::{
    ALWAYS_SUCCESS_TYPE_ARGS, FAKE_XUDT_OWNER_TYPE_ID, OWNER_LOCK_ARGS_1, OWNER_LOCK_ARGS_2, TYPE_ID_ARGS,
    XUDT_RCE_TYPE_ARGS,
};

#[test]
fn test_token_cell_transfer_token() {
    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "always_success",
                "type_args": ALWAYS_SUCCESS_TYPE_ARGS
            },
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "xudt_rce",
                "type_args": XUDT_RCE_TYPE_ARGS
            },
        ],
        "inputs": [
            {
                "tmp_type": "XudtCell",
                "previous_output": {
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{always_success}}",
                        "args": OWNER_LOCK_ARGS_1
                    },
                    "type": {
                        "code_hash": "{{xudt_rce}}",
                        "args": {
                            "owner_script_hash": {
                                "code_hash": FAKE_XUDT_OWNER_TYPE_ID,
                            },
                            "xudt_args": {
                                "flag": "1",
                                "ScriptVec":[
                                    {
                                        "code_hash": "{{xudt_extension.so}}",
                                        "hash_type": "type",
                                        "args": TYPE_ID_ARGS
                                    }
                                ]
                            },
                        }
                    },
                    "tmp_data": {
                        "amount" :"1000"
                    }
                },
            },
            {
                "tmp_type": "XudtCell",
                "previous_output": {
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{always_success}}",
                        "args": OWNER_LOCK_ARGS_1
                    },
                    "type": {
                        "code_hash": "{{xudt_rce}}",
                        "args": {
                            "owner_script_hash": {
                                "code_hash": FAKE_XUDT_OWNER_TYPE_ID,
                            },
                            "xudt_args": {
                                "flag": "1",
                                "ScriptVec":[
                                    {
                                        "code_hash": "{{xudt_extension.so}}",
                                        "hash_type": "type",
                                        "args": TYPE_ID_ARGS
                                    }
                                ]
                            },
                        }
                    },
                    "tmp_data": {
                        "amount" :"1000"
                    }
                },
            }
        ],
        "outputs": [
            {
                "tmp_type": "XudtCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{always_success}}",
                    "args": OWNER_LOCK_ARGS_2
                },
                "type": {
                    "code_hash": "{{xudt_rce}}",
                    "args": {
                        "owner_script_hash": {
                            "code_hash": FAKE_XUDT_OWNER_TYPE_ID,
                        },
                        "xudt_args": {
                            "flag": 1,
                            "ScriptVec":[
                                {
                                    "code_hash": "{{xudt_extension.so}}",
                                    "hash_type": "type",
                                    "args": TYPE_ID_ARGS
                                }
                            ]
                        },
                    }
                },
                "tmp_data": {
                    "amount": "1000"
                }
            },
            {
                "tmp_type": "XudtCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{always_success}}",
                    "args": OWNER_LOCK_ARGS_2
                },
                "type": {
                    "code_hash": "{{xudt_rce}}",
                    "args": {
                        "owner_script_hash": {
                            "code_hash": FAKE_XUDT_OWNER_TYPE_ID,
                        },
                        "xudt_args": {
                            "flag": 1,
                            "ScriptVec":[
                                {
                                    "code_hash": "{{xudt_extension.so}}",
                                    "hash_type": "type",
                                    "args": TYPE_ID_ARGS
                                }
                            ]
                        },
                    }
                },
                "tmp_data": {
                    "amount": "1000"
                }
            }
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}
