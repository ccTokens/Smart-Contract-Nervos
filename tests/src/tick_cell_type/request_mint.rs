use serde_json::json;

use crate::util;
use crate::util::constants::{
    ALWAYS_SUCCESS_TYPE_ARGS, CONFIG_CELL_TYPE_ARGS, DUMMY_TX_HASH, FAKE_OMNI_LOCK_TYPE_ARGS,
    GOVERNANCE_MEMBER_CELL_TYPE_ARGS, MERCHANT_LOCK_ARGS_1, MERCHANT_LOCK_ARGS_2, MERCHANT_LOCK_ARGS_3,
    MERCHANT_LOCK_ARGS_4, MERCHANT_LOCK_ARGS_5, OWNER_LOCK_ARGS_1, TICK_CELL_TYPE_ARGS, TYPE_ID_ARGS,
};

#[test]
fn test_tick_cell_request_mint() {
    let tx = json!({
        "cell_deps": [
            {
                "tmp_type": "deployed_contract",
                "tmp_file_name": "always_success",
                "type_args": ALWAYS_SUCCESS_TYPE_ARGS
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

        ],
        "inputs": [
            {
                "previous_output": {
                    "lock": {
                        "code_hash": "{{fake-omni-lock}}",
                        "args": MERCHANT_LOCK_ARGS_1
                    },
                }
            }
        ],
        "outputs": [
            {
                "tmp_type": "TickCell",
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
                        "token_id": TYPE_ID_ARGS,
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
            }
        ],
        "witnesses":[
            "0x",
            util::gen_action(0, "request_mint"),
        ]
    });

    let mut template_parser = util::init_template_parser();
    template_parser.parse_and_verify(tx, u64::MAX, None)
}
