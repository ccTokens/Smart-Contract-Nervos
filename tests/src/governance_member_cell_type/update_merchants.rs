use serde_json::json;

use crate::util;
use crate::util::constants::{
    ExpectedError, ALWAYS_SUCCESS_TYPE_ARGS, CONFIG_CELL_TYPE_ARGS, CUSTODIAN_LOCK_ARGS_1, CUSTODIAN_LOCK_ARGS_2,
    CUSTODIAN_LOCK_ARGS_3, CUSTODIAN_LOCK_ARGS_4, CUSTODIAN_LOCK_ARGS_5, FAKE_OMNI_LOCK_TYPE_ARGS,
    GOVERNANCE_MEMBER_CELL_TYPE_ARGS, MERCHANT_LOCK_ARGS_1, MERCHANT_LOCK_ARGS_2, MERCHANT_LOCK_ARGS_3,
    MERCHANT_LOCK_ARGS_4, MERCHANT_LOCK_ARGS_5, OWNER_LOCK_ARGS_1,
};

#[test]
fn test_governance_member_update_merchants() {
    let custodian_lock_args = util::gen_custodian_lock_args();
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
                        "multisig_args": {
                            "require_first_n": 0,
                            "threshold": 3,
                        },
                        "members": [
                            CUSTODIAN_LOCK_ARGS_1,
                            CUSTODIAN_LOCK_ARGS_2,
                            CUSTODIAN_LOCK_ARGS_3,
                            CUSTODIAN_LOCK_ARGS_4,
                            CUSTODIAN_LOCK_ARGS_5,
                        ]
                    }
                }
            },
        ],
        "inputs": [
            {
                "previous_output": {
                    "tmp_type": "GovernanceMemberCell",
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{always_success}}",
                    },
                    "type": {
                        "code_hash": "{{governance-member-cell-type}}",
                        "args": {
                            "role": "merchant",
                            "cell_id": "0x0000000000000000000000000000000000000000000000000000000000000002"
                        }
                    },
                    "tmp_data": {
                        "version": 0,
                        "parent_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                        "members": [
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_1),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_2),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_3),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_4),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_5),
                        ]
                    }
                }
            },
            {
                "previous_output": {
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": custodian_lock_args
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
                        "cell_id": "0x0000000000000000000000000000000000000000000000000000000000000002"
                    }
                },
                "tmp_data": {
                    "version": 0,
                    "parent_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                    "members": [
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_1),
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_2),
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_3),
                        util::gen_merchant_script("0xEE00000000000000000000000000000000000006"),
                        util::gen_merchant_script("0xEE00000000000000000000000000000000000007"),
                    ]
                }
            },
            {
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": custodian_lock_args
                }
            }
        ],
        "witnesses": [
            "0x",
            "0x",
            util::gen_action(0, "update_merchants"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}

#[test]
fn challenge_governance_member_update_merchants_with_custodian_lock() {
    let custodian_lock_args = util::gen_custodian_lock_args();
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
                        "multisig_args": {
                            "require_first_n": 0,
                            "threshold": 3,
                        },
                        "members": [
                            CUSTODIAN_LOCK_ARGS_1,
                            CUSTODIAN_LOCK_ARGS_2,
                            CUSTODIAN_LOCK_ARGS_3,
                            CUSTODIAN_LOCK_ARGS_4,
                            CUSTODIAN_LOCK_ARGS_5,
                        ]
                    }
                }
            },
        ],
        "inputs": [
            {
                "previous_output": {
                    "tmp_type": "GovernanceMemberCell",
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{always_success}}",
                    },
                    "type": {
                        "code_hash": "{{governance-member-cell-type}}",
                        "args": {
                            "role": "merchant",
                            "cell_id": "0x0000000000000000000000000000000000000000000000000000000000000002"
                        }
                    },
                    "tmp_data": {
                        "version": 0,
                        "parent_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                        "members": [
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_1),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_2),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_3),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_4),
                            util::gen_merchant_script(MERCHANT_LOCK_ARGS_5),
                        ]
                    }
                }
            },
            {
                "previous_output": {
                    "capacity": 0,
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": custodian_lock_args
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
                        "cell_id": "0x0000000000000000000000000000000000000000000000000000000000000002"
                    }
                },
                "tmp_data": {
                    "version": 0,
                    "parent_id": "0x0000000000000000000000000000000000000000000000000000000000000001",
                    "members": [
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_1),
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_2),
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_3),
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_4),
                        util::gen_merchant_script(MERCHANT_LOCK_ARGS_5),
                        // Simulate register the custodian as merchant
                        util::gen_merchant_script(&util::gen_custodian_lock_args()),
                    ]
                }
            },
            {
                "capacity": 0,
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": custodian_lock_args
                }
            }
        ],
        "witnesses": [
            "0x",
            "0x",
            util::gen_action(0, "update_merchants"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, Some(ExpectedError::CustodianLockMustNotInMerchants as i8))
}
