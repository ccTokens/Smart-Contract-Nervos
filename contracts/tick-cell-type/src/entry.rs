use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::{format, vec};
use core::cmp::Ordering;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::prelude::*;
use ckb_std::high_level;
use contract_core::config::{check_system_status, get_config_by_key};
use contract_core::constants::ScriptType;
use contract_core::error::{AsI8, CoreError};
use contract_core::util::{self};
use contract_core::{cc_assert, debug, verifiers};
use types::constants::Action::{self};
use types::constants::{ConfigKey, TickType, TOKEN_ID_SIZE};
use types::packed::{Byte32, Script};

use crate::error::TickError;
use crate::parser::{parse_tick, TickCellData};

pub fn main() -> Result<(), Box<dyn AsI8>> {
    debug!("====== Running tick-cell-type ======");

    let self_script = Script::from(high_level::load_script().map_err(TickError::from)?);
    let (input_tick_cells, output_tick_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, self_script.as_reader())?;
    let xudt_type_id = match Byte32::try_from(get_config_by_key(ConfigKey::XudtCellTypeId)?) {
        Ok(data) => data,
        Err(_) => {
            return Err(TickError::Encoding.into());
        }
    };
    let (input_xudt_cells, output_xudt_cells) =
        util::find_cells_by_type_id_in_inputs_and_outputs(ScriptType::Type, xudt_type_id.as_reader())?;

    let action = util::get_tx_action()?;

    debug!("==== Action {} ====", action.to_string());

    match action {
        Action::RequestMint => request(input_tick_cells, output_tick_cells, TickType::Mint)?,
        Action::ConfirmMint => confirm_mint(input_tick_cells, output_tick_cells, input_xudt_cells, output_xudt_cells)?,
        Action::RejectMint => reject_mint(input_tick_cells, output_tick_cells, input_xudt_cells, output_xudt_cells)?,
        Action::RequestBurn => request(input_tick_cells, output_tick_cells, TickType::Burn)?,
        Action::ConfirmBurn => confirm_burn(input_tick_cells, output_tick_cells, input_xudt_cells, output_xudt_cells)?,
        Action::RejectBurn => reject_burn(input_tick_cells, output_tick_cells, input_xudt_cells, output_xudt_cells)?,
        _ => {
            return Err(CoreError::ActionNotSupported {
                action: action.to_string(),
            }
            .into());
        }
    }

    Ok(())
}

fn request(
    input_tick_cells: Vec<usize>,
    output_tick_cells: Vec<usize>,
    tick_type: TickType,
) -> Result<(), Box<dyn AsI8>> {
    check_system_status()?;

    verifiers::permission::verify_input_has_merchant_lock(0)?;

    verifiers::basic::verify_cell_number_and_position("TickCell", &input_tick_cells, &[], &output_tick_cells, &[0])?;

    let tick = load_tick_data(0, Source::Output)?;

    verifiers::permission::verify_cell_has_always_success_lock(0, Source::Output)?;

    verify_if_tick_data_valid(tick_type, &tick)?;
    verify_if_tick_belong_to_merchant(&tick.merchant, 0, Source::Input)?;

    Ok(())
}

fn confirm_mint(
    input_tick_cells: Vec<usize>,
    output_tick_cells: Vec<usize>,
    input_xudt_cells: Vec<usize>,
    output_xudt_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    verifiers::permission::verify_input_has_custodian_lock(1)?;
    verifiers::basic::verify_cell_number_and_position("TickCell", &input_tick_cells, &[0], &output_tick_cells, &[])?;
    verifiers::basic::verify_cell_number_range(
        "XudtCell",
        &input_xudt_cells,
        (Ordering::Equal, 0),
        &output_xudt_cells,
        (Ordering::Greater, 0),
    )?;

    let tick = load_tick_data(0, Source::Input)?;

    cc_assert!(
        tick.type_ == TickType::Mint,
        TickError::InvalidTickType {
            current: tick.type_.to_string(),
            expected: TickType::Mint.to_string()
        }
    );

    let (token_id, xudt_amount_map) = collect_xudt_map(output_xudt_cells, Source::Output)?;

    cc_assert!(
        token_id == tick.token_id,
        TickError::XudtCellTokenIdMismatch {
            expected: hex::encode(&tick.token_id),
            current: hex::encode(&token_id),
        }
    );

    let merchant_lock = tick.merchant.as_slice();
    match xudt_amount_map.get(merchant_lock) {
        Some(amount) => {
            cc_assert!(
                *amount == tick.value,
                TickError::XudtTransferError {
                    target_lock: format!("{}", tick.merchant),
                    token_id: hex::encode(&tick.token_id),
                    amount: tick.value,
                }
            );
        }
        None => {
            return Err(TickError::XudtTransferError {
                target_lock: format!("{}", tick.merchant),
                token_id: hex::encode(&tick.token_id),
                amount: tick.value,
            }
            .into());
        }
    }

    Ok(())
}

fn reject_mint(
    input_tick_cells: Vec<usize>,
    output_tick_cells: Vec<usize>,
    input_xudt_cells: Vec<usize>,
    output_xudt_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    verifiers::permission::verify_input_has_custodian_lock(1)?;
    verifiers::basic::verify_cell_number_and_position("TickCell", &input_tick_cells, &[0], &output_tick_cells, &[])?;
    verifiers::basic::verify_cell_number_range(
        "TickCell",
        &input_xudt_cells,
        (Ordering::Equal, 0),
        &output_xudt_cells,
        (Ordering::Equal, 0),
    )?;

    let tick = load_tick_data(0, Source::Input)?;

    cc_assert!(
        tick.type_ == TickType::Mint,
        TickError::InvalidTickType {
            current: tick.type_.to_string(),
            expected: TickType::Mint.to_string()
        }
    );

    Ok(())
}

fn confirm_burn(
    input_tick_cells: Vec<usize>,
    output_tick_cells: Vec<usize>,
    input_xudt_cells: Vec<usize>,
    output_xudt_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    verifiers::permission::verify_input_has_custodian_lock(1)?;
    verifiers::basic::verify_cell_number_and_position("TickCell", &input_tick_cells, &[0], &output_tick_cells, &[])?;
    verifiers::basic::verify_cell_number_range(
        "XudtCell",
        &input_xudt_cells,
        (Ordering::Greater, 0),
        // The output_xudt_cells could be empty or have some change cells.
        &[],
        (Ordering::Equal, 0),
    )?;

    let tick = load_tick_data(0, Source::Input)?;

    cc_assert!(
        tick.type_ == TickType::Burn,
        TickError::InvalidTickType {
            current: tick.type_.to_string(),
            expected: TickType::Burn.to_string()
        }
    );

    let (input_token_id, input_xudt_amount_map) = collect_xudt_map(input_xudt_cells, Source::Input)?;

    cc_assert!(
        input_token_id == tick.token_id,
        TickError::XudtCellTokenIdMismatch {
            expected: hex::encode(&tick.token_id),
            current: hex::encode(&input_token_id),
        }
    );

    let total_input_amount: u128 = input_xudt_amount_map.iter().map(|(_, v)| v.to_owned()).sum();

    let total_output_amount = if output_xudt_cells.len() == 0 {
        0u128
    } else {
        let (output_token_id, output_xudt_amount_map) = collect_xudt_map(output_xudt_cells, Source::Output)?;

        cc_assert!(
            output_token_id == tick.token_id,
            TickError::XudtCellTokenIdMismatch {
                expected: hex::encode(&tick.token_id),
                current: hex::encode(&output_token_id),
            }
        );

        output_xudt_amount_map.iter().map(|(_, v)| v.to_owned()).sum()
    };

    cc_assert!(
        total_input_amount == total_output_amount + tick.value,
        TickError::BurnedXudtAmountNotMatch {
            burned: if total_input_amount > total_output_amount {
                total_input_amount - total_output_amount
            } else {
                0
            },
            expected: tick.value
        }
    );

    Ok(())
}

fn reject_burn(
    input_tick_cells: Vec<usize>,
    output_tick_cells: Vec<usize>,
    input_xudt_cells: Vec<usize>,
    output_xudt_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    verifiers::permission::verify_input_has_custodian_lock(1)?;
    verifiers::basic::verify_cell_number_and_position("TickCell", &input_tick_cells, &[0], &output_tick_cells, &[])?;
    verifiers::basic::verify_cell_number_range(
        "XudtCell",
        &input_xudt_cells,
        (Ordering::Greater, 0),
        &output_xudt_cells,
        (Ordering::Greater, 0),
    )?;

    let tick = load_tick_data(0, Source::Input)?;

    cc_assert!(
        tick.type_ == TickType::Burn,
        TickError::InvalidTickType {
            current: tick.type_.to_string(),
            expected: TickType::Burn.to_string()
        }
    );

    let merchant_lock = tick.merchant.as_slice();
    let (token_id, xudt_amount_map) = collect_xudt_map(output_xudt_cells, Source::Output)?;

    cc_assert!(
        token_id == tick.token_id,
        TickError::XudtCellTokenIdMismatch {
            expected: hex::encode(&tick.token_id),
            current: hex::encode(&token_id),
        }
    );

    match xudt_amount_map.get(merchant_lock) {
        Some(amount) => {
            cc_assert!(
                *amount >= tick.value,
                TickError::XudtTransferError {
                    target_lock: format!("{}", tick.merchant),
                    token_id: hex::encode(&tick.token_id),
                    amount: tick.value,
                }
            );
        }
        None => {
            return Err(TickError::XudtTransferError {
                target_lock: format!("{}", tick.merchant),
                token_id: hex::encode(&tick.token_id),
                amount: tick.value,
            }
            .into());
        }
    }

    Ok(())
}

fn verify_if_tick_data_valid(expected_type: TickType, tick: &TickCellData) -> Result<(), Box<dyn AsI8>> {
    debug!("Verify if the fields of TickCell is valid.");

    cc_assert!(
        tick.type_ == expected_type,
        TickError::InvalidTickType {
            current: tick.type_.to_string(),
            expected: TickType::Burn.to_string()
        }
    );
    cc_assert!(tick.token_id.len() == TOKEN_ID_SIZE, TickError::InvalidTickTokenIdSize);
    cc_assert!(tick.value > 0, TickError::TickValueCanNotBeZero);

    Ok(())
}

fn load_tick_data(index: usize, source: Source) -> Result<TickCellData, Box<dyn AsI8>> {
    let data = high_level::load_cell_data(index, source).map_err(TickError::from)?;
    let tick_data = parse_tick(&data)?;

    Ok(tick_data)
}

fn verify_if_tick_belong_to_merchant(
    tick_merchant: &Script,
    index: usize,
    source: Source,
) -> Result<(), Box<dyn AsI8>> {
    let sign_merchant = high_level::load_cell_lock(index, source).map_err(TickError::from)?;

    cc_assert!(
        util::is_entity_eq(tick_merchant, &sign_merchant),
        TickError::InvalidTickMerchantLock {
            lock: format!("{}", sign_merchant)
        }
    );

    Ok(())
}

fn collect_xudt_map(
    xudt_cells: Vec<usize>,
    source: Source,
) -> Result<(Vec<u8>, BTreeMap<Vec<u8>, u128>), Box<dyn AsI8>> {
    debug!("Collecting information of XudtCells ...");

    let mut token_id = vec![];
    let mut has_found_args = vec![];
    let mut xudt_amount_map = BTreeMap::new();
    for index in xudt_cells {
        let type_script = match high_level::load_cell_type(index, source).map_err(TickError::from)? {
            Some(data) => data,
            None => unreachable!(),
        };
        let args = type_script.as_reader().args().raw_data();

        debug!("The current XudtCell.type.args is: {}", hex::encode(args));

        if has_found_args.is_empty() {
            has_found_args = args.to_vec();
        } else {
            cc_assert!(
                has_found_args == args,
                TickError::MultipleKindOfXudtFound {
                    index,
                    source: format!("{:?}", source)
                }
            );
        }

        token_id = (&args[..32]).to_vec();

        debug!("The xudt token ID is: {}", hex::encode(&token_id));

        let data = high_level::load_cell_data(index, source).map_err(TickError::from)?;

        cc_assert!(
            data.len() >= 16, // 16 is the size of u128
            TickError::UnsupportedXudtData {
                index,
                source: format!("{:?}", source)
            }
        );

        let lock_script = high_level::load_cell_lock(index, source).map_err(TickError::from)?;
        let key = lock_script.as_slice().to_vec();
        let mut amount = u128::from_le_bytes((&data[..16]).try_into().unwrap());

        amount = match xudt_amount_map.get(&key) {
            Some(prev_amount) => *prev_amount + amount,
            None => amount,
        };

        xudt_amount_map.insert(key, amount);

        debug!(
            "Found XudtCell with lock({}) have amount({}) token({})",
            lock_script,
            amount,
            hex::encode(&token_id)
        );
    }

    Ok((token_id, xudt_amount_map))
}
