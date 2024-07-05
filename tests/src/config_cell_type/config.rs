use serde_json::json;
use types::constants::ConfigKey;

use crate::util;
use crate::util::constants::{
    ExpectedError, CONFIG_CELL_TYPE_ARGS, DEPLOY_LOCK_ARGS, FAKE_OMNI_LOCK_TYPE_ARGS,
    FAKE_SECPK1_BLAKE160_SIGNHASH_ALL_ARGS, OWNER_LOCK_ARGS_1,
};

#[test]
fn test_config_deploy() {
    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "fake-omni-lock",
                "type_args": FAKE_OMNI_LOCK_TYPE_ARGS,
            },
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "fake-secp256k1-blake160-signhash-all",
                "type_args": FAKE_SECPK1_BLAKE160_SIGNHASH_ALL_ARGS,
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
        ],
        "inputs": [
            {
                "previous_output": {
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": OWNER_LOCK_ARGS_1
                    }
                },
            }
        ],
        "outputs": [
            {
                "tmp_type": "ConfigCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": OWNER_LOCK_ARGS_1
                },
                "type": {
                    "code_hash": "{{config-cell-type}}",
                },
                "tmp_data": {
                    "version": 0,
                    "configs": util::gen_configs()
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "deploy_config"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}

#[test]
fn test_config_update() {
    let mut configs = util::gen_configs();
    // Modify the system status to 0x00
    configs[0] = json!([ConfigKey::SystemStatus as u32, "0x00"]);

    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "fake-omni-lock",
                "type_args": FAKE_OMNI_LOCK_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
        ],
        "inputs": [
            {
                "previous_output": {
                    "tmp_type": "ConfigCell",
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": OWNER_LOCK_ARGS_1
                    },
                    "type": {
                        "code_hash": "{{config-cell-type}}",
                    },
                    "tmp_data": {
                        "version": 0,
                        "configs": [
                            [ConfigKey::SystemStatus as u32, "0x00"],
                        ]
                    }
                },
            }
        ],
        "outputs": [
            {
                "tmp_type": "ConfigCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": OWNER_LOCK_ARGS_1
                },
                "type": {
                    "code_hash": "{{config-cell-type}}",
                },
                "tmp_data": {
                    "version": 0,
                    "configs": configs
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "update_config"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}

#[test]
fn challenge_config_deploy_without_owner_lock() {
    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "fake-omni-lock",
                "type_args": FAKE_OMNI_LOCK_TYPE_ARGS,
            },
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "fake-secp256k1-blake160-signhash-all",
                "type_args": FAKE_SECPK1_BLAKE160_SIGNHASH_ALL_ARGS,
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
        ],
        "inputs": [
            {
                "previous_output": {
                    "capacity": 0,
                    "lock": {
                        // Simulate not using the owner lock
                        "code_hash": "{{fake-secp256k1-blake160-signhash-all}}",
                        "args": DEPLOY_LOCK_ARGS
                    }
                },
            }
        ],
        "outputs": [
            {
                "tmp_type": "ConfigCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": OWNER_LOCK_ARGS_1
                },
                "type": {
                    "code_hash": "{{config-cell-type}}",
                },
                "tmp_data": {
                    "version": 0,
                    "configs": util::gen_configs()
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "deploy_config"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, Some(ExpectedError::OwnerLockIsRequired as i8))
}
