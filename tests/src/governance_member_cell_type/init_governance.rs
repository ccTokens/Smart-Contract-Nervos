use serde_json::json;

use crate::util;
use crate::util::constants::{
    ExpectedError, ALWAYS_SUCCESS_TYPE_ARGS, CONFIG_CELL_TYPE_ARGS, DEPLOY_LOCK_ARGS, FAKE_OMNI_LOCK_TYPE_ARGS,
    FAKE_SECPK1_BLAKE160_SIGNHASH_ALL_ARGS, GOVERNANCE_MEMBER_CELL_TYPE_ARGS, OWNER_LOCK_ARGS_1,
};

#[test]
fn test_governance_member_init_custodian() {
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
                "tmp_file_name": "governance-member-cell-type",
                "type_args": GOVERNANCE_MEMBER_CELL_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
            {
                "out_point": {
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
                }
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
                "tmp_type": "GovernanceMemberCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": OWNER_LOCK_ARGS_1
                },
                "type": {
                    "code_hash": "{{governance-member-cell-type}}",
                    "args": {
                        "role": "custodian",
                        "cell_id": "{{type-id}}"
                    }
                },
                "tmp_data": {
                    "version": 0,
                    "multisig_args": {
                        "require_first_n": 0,
                        "threshold": 3,
                    },
                    "members": [
                        "0xFF00000000000000000000000000000000000001",
                        "0xFF00000000000000000000000000000000000002",
                        "0xFF00000000000000000000000000000000000003",
                        "0xFF00000000000000000000000000000000000004",
                        "0xFF00000000000000000000000000000000000005",
                    ]
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "init_governance"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}

#[test]
fn test_governance_member_init_merchant() {
    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "always_success",
                "type_args": ALWAYS_SUCCESS_TYPE_ARGS,
            },
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
                "tmp_file_name": "governance-member-cell-type",
                "type_args": GOVERNANCE_MEMBER_CELL_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
            {
                "out_point": {
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
                }
            },
            {
                "out_point": {
                    "tmp_type": "GovernanceMemberCell",
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": OWNER_LOCK_ARGS_1
                    },
                    "type": {
                        "code_hash": "{{governance-member-cell-type}}",
                        "args": {
                            "role": "custodian",
                            "cell_id": "0x0000000000000000000000000000000000000000000000000000000000000001"
                        }
                    },
                    "tmp_data": {
                        "version": 0,
                        // "parent_id": null,
                        "multisig_args": {
                            "require_first_n": 0,
                            "threshold": 3,
                        },
                        "members": [
                            "0xFF00000000000000000000000000000000000001",
                            "0xFF00000000000000000000000000000000000002",
                            "0xFF00000000000000000000000000000000000003",
                            "0xFF00000000000000000000000000000000000004",
                            "0xFF00000000000000000000000000000000000005",
                        ]
                    }
                }
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
                "tmp_type": "GovernanceMemberCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{always_success}}",
                },
                "type": {
                    "code_hash": "{{governance-member-cell-type}}",
                    "args": {
                        "role": "merchant",
                        "cell_id": "{{type-id}}"
                    }
                },
                "tmp_data": {
                    "version": 0,
                    "parent_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                    "members": [
                        "0xEE00000000000000000000000000000000000001",
                        "0xEE00000000000000000000000000000000000002",
                        "0xEE00000000000000000000000000000000000003",
                        "0xEE00000000000000000000000000000000000004",
                        "0xEE00000000000000000000000000000000000005",
                    ]
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "init_governance"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}

#[test]
fn chanllenge_governance_member_init_custodian_without_owner_lock() {
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
                "tmp_file_name": "governance-member-cell-type",
                "type_args": GOVERNANCE_MEMBER_CELL_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
            {
                "out_point": {
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
                }
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
                "tmp_type": "GovernanceMemberCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": OWNER_LOCK_ARGS_1
                },
                "type": {
                    "code_hash": "{{governance-member-cell-type}}",
                    "args": {
                        "role": "custodian",
                        "cell_id": "{{type-id}}"
                    }
                },
                "tmp_data": {
                    "version": 0,
                    "multisig_args": {
                        "require_first_n": 0,
                        "threshold": 3,
                    },
                    "members": [
                        "0xFF00000000000000000000000000000000000001",
                        "0xFF00000000000000000000000000000000000002",
                        "0xFF00000000000000000000000000000000000003",
                        "0xFF00000000000000000000000000000000000004",
                        "0xFF00000000000000000000000000000000000005",
                    ]
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "init_governance"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, Some(ExpectedError::OwnerLockIsRequired as i8))
}

#[test]
fn challenge_governance_member_init_merchant_without_owner_lock() {
    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "always_success",
                "type_args": ALWAYS_SUCCESS_TYPE_ARGS,
            },
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
                "tmp_file_name": "governance-member-cell-type",
                "type_args": GOVERNANCE_MEMBER_CELL_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
            {
                "out_point": {
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
                }
            },
            {
                "out_point": {
                    "tmp_type": "GovernanceMemberCell",
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": OWNER_LOCK_ARGS_1
                    },
                    "type": {
                        "code_hash": "{{governance-member-cell-type}}",
                        "args": {
                            "role": "custodian",
                            "cell_id": "0x0000000000000000000000000000000000000000000000000000000000000001"
                        }
                    },
                    "tmp_data": {
                        "version": 0,
                        // "parent_id": null,
                        "multisig_args": {
                            "require_first_n": 0,
                            "threshold": 3,
                        },
                        "members": [
                            "0xFF00000000000000000000000000000000000001",
                            "0xFF00000000000000000000000000000000000002",
                            "0xFF00000000000000000000000000000000000003",
                            "0xFF00000000000000000000000000000000000004",
                            "0xFF00000000000000000000000000000000000005",
                        ]
                    }
                }
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
                "tmp_type": "GovernanceMemberCell",
                "capacity": 0,
                "lock": {
                    "code_hash": "{{always_success}}",
                },
                "type": {
                    "code_hash": "{{governance-member-cell-type}}",
                    "args": {
                        "role": "merchant",
                        "cell_id": "{{type-id}}"
                    }
                },
                "tmp_data": {
                    "version": 0,
                    "parent_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                    "members": [
                        "0xEE00000000000000000000000000000000000001",
                        "0xEE00000000000000000000000000000000000002",
                        "0xEE00000000000000000000000000000000000003",
                        "0xEE00000000000000000000000000000000000004",
                        "0xEE00000000000000000000000000000000000005",
                    ]
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "init_governance"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, Some(ExpectedError::OwnerLockIsRequired as i8))
}
