use serde_json::json;

use crate::util;
use crate::util::constants::{
    CONFIG_CELL_TYPE_ARGS, FAKE_OMNI_LOCK_TYPE_ARGS, GOVERNANCE_MEMBER_CELL_TYPE_ARGS, OWNER_LOCK_ARGS_1,
};

#[test]
fn test_governance_member_update_custodians() {
    let tx = json!({
        "cell_deps": [
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
        ],
        "inputs": [
            {
                "previous_output": {
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
                            "0xFF00000000000000000000000000000000000001",
                            "0xFF00000000000000000000000000000000000002",
                            "0xFF00000000000000000000000000000000000003",
                            "0xFF00000000000000000000000000000000000004",
                            "0xFF00000000000000000000000000000000000005",
                        ]
                    }
                }
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
                        "0xFF00000000000000000000000000000000000003",
                        "0xFF00000000000000000000000000000000000004",
                        "0xFF00000000000000000000000000000000000005",
                        // New custodians
                        "0xFF00000000000000000000000000000000000006",
                        "0xFF00000000000000000000000000000000000007",
                    ]
                }
            },
        ],
        "witnesses": [
            "0x",
            util::gen_action(0, "update_custodians"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}
