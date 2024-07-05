use serde_json::json;

use super::common::{gen_xudt_args, gen_xudt_token_id, gen_xudt_witness};
use crate::util;
use crate::util::constants::{
    ALWAYS_SUCCESS_TYPE_ARGS, CONFIG_CELL_TYPE_ARGS, CUSTODIAN_LOCK_ARGS_1, CUSTODIAN_LOCK_ARGS_2,
    CUSTODIAN_LOCK_ARGS_3, CUSTODIAN_LOCK_ARGS_4, CUSTODIAN_LOCK_ARGS_5, DUMMY_TX_HASH, FAKE_OMNI_LOCK_TYPE_ARGS,
    GOVERNANCE_MEMBER_CELL_TYPE_ARGS, MERCHANT_LOCK_ARGS_1, OWNER_LOCK_ARGS_1, TICK_CELL_TYPE_ARGS,
    XUDT_OWNER_TYPE_ARGS, XUDT_RCE_TYPE_ARGS,
};

#[test]
fn test_tick_cell_confirm_mint() {
    let custodian_lock_args = util::gen_custodian_lock_args();
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
            {
                "tmp_type": "contract",
                "tmp_file_name": "xudt_owner.so",
                "type_args": XUDT_OWNER_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "tick-cell-type",
                "type_args": TICK_CELL_TYPE_ARGS
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "governance-member-cell-type",
                "type_args": GOVERNANCE_MEMBER_CELL_TYPE_ARGS
            },
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "fake-omni-lock",
                "type_args": FAKE_OMNI_LOCK_TYPE_ARGS,
            },
            {
                "tmp_type": "contract",
                "tmp_file_name": "config-cell-type",
                "type_args": CONFIG_CELL_TYPE_ARGS
            },
            {
                "out_point": {
                    "tmp_type": "ConfigCell",
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
                "tmp_type": "TickCell",
                "previous_output": {
                    "lock": {
                        "code_hash": "{{always_success}}",
                    },
                    "type": {
                        "code_hash": "{{tick-cell-type}}",
                    },
                    "tmp_data": {
                        "version": 0,
                        "Tick":{
                            "tick_type": "mint",
                            "token_id": gen_xudt_token_id(),
                            "value": "1000",
                            "merchant": {
                                "code_hash": "{{fake-omni-lock}}",
                                "args": MERCHANT_LOCK_ARGS_1,
                            },
                            "coin_type": "0x80000001",
                            "tx_hash": DUMMY_TX_HASH,
                            "receipt_addr": "bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxgxxxx",
                        }
                    }
                },
            },
            {
                "previous_output": {
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": custodian_lock_args
                    },
                }
            }
        ],
        "outputs": [
            {
                "tmp_type": "XudtCell",
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": MERCHANT_LOCK_ARGS_1,
                },
                "type": {
                    "code_hash": "{{xudt_rce}}",
                    "args": gen_xudt_args()
                },
                "tmp_data": {
                    "amount": 1000
                }
            },
            {
                "lock": {
                    "code_hash": "{{fake-omni-lock}}",
                    "args": custodian_lock_args
                },
            }
        ],
        "witnesses":[
            {
                "tmp_type": "xudt",
                "lock": "0x",
                "output_type": gen_xudt_witness()
            },
            "0x",
            util::gen_action(0, "confirm_mint"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}
