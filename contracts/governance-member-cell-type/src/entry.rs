use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::result::Result;

use ckb_std::ckb_constants::Source;
use ckb_std::{debug, high_level};
use contract_core::config::always_success_lock;
use contract_core::constants::{CellField, ScriptType};
use contract_core::data_parser::governance_member_cell;
use contract_core::error::{AsI8, CoreError};
use contract_core::{cc_assert, util, verifiers};
use types::constants::{owner_lock, Action, GovernanceMemberRole};
use types::packed::Script;
use types::prelude::{Entity, Reader};

use super::error::GovernanceError;

pub fn main() -> Result<(), Box<dyn AsI8>> {
    debug!("====== Running governance-member-cell-type ======");

    let self_script = Script::from(high_level::load_script().map_err(GovernanceError::from)?);
    let (input_governance_cells, output_governance_cells) =
        util::find_cells_by_script_in_inputs_and_outputs(ScriptType::Type, self_script.as_reader())?;

    let action = util::get_tx_action()?;

    debug!("==== Action {} ====", action.to_string());

    match action {
        Action::InitGovernance => init_governance(input_governance_cells, output_governance_cells)?,
        Action::UpdateOwner => update_owner(input_governance_cells, output_governance_cells)?,
        Action::UpdateCustodians => update_custodians(input_governance_cells, output_governance_cells)?,
        Action::UpdateMerchants => update_merchants(input_governance_cells, output_governance_cells)?,
        _ => {
            return Err(CoreError::ActionNotSupported {
                action: action.to_string(),
            }
            .into());
        }
    }

    Ok(())
}

fn init_governance(
    input_governance_cells: Vec<usize>,
    output_governance_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    verifiers::permission::verify_input_has_owner_lock(0)?;

    verifiers::basic::verify_cell_number_and_position(
        "GovernanceMemberCell",
        &input_governance_cells,
        &[],
        &output_governance_cells,
        &[0],
    )?;

    let (role, cell_id) = util::load_governance_member_type_info(output_governance_cells[0], Source::Output)?;

    verify_cell_id_correct(&cell_id, output_governance_cells[0])?;

    match role {
        GovernanceMemberRole::Custodian => {
            verify_the_custodian_cell_lock(output_governance_cells[0])?;
            verify_the_custodian_cell_data(output_governance_cells[0])?;
        }
        GovernanceMemberRole::Merchant => {
            debug!("Verify if the GovernanceMemberCell in outputs has always_success lock.");

            let lock = high_level::load_cell_lock(output_governance_cells[0], Source::Output)
                .map_err(GovernanceError::from)?;
            let always_success_lock = always_success_lock()?;
            cc_assert!(
                util::is_entity_eq(&always_success_lock, &lock),
                GovernanceError::NewCellLockError {
                    current: lock.to_string(),
                    expected: always_success_lock.to_string()
                }
            );

            verify_the_merchant_cell_data(output_governance_cells[0])?;
        }
    }

    Ok(())
}

fn update_owner(input_governance_cells: Vec<usize>, output_governance_cells: Vec<usize>) -> Result<(), Box<dyn AsI8>> {
    verifiers::basic::verify_cell_number_and_position(
        "GovernanceMemberCell",
        &input_governance_cells,
        &[0],
        &output_governance_cells,
        &[0],
    )?;

    verify_the_role_of_target(
        input_governance_cells[0],
        GovernanceMemberRole::Custodian,
        "Only the custodian can update its owner.".to_string(),
    )?;

    // Only lock field can be updated
    verifiers::basic::verify_cell_consistent_with_exception(
        "GovernanceMemberCell",
        input_governance_cells[0],
        output_governance_cells[0],
        vec![CellField::Lock],
    )?;

    verify_the_custodian_cell_lock(output_governance_cells[0])?;

    Ok(())
}

fn update_custodians(
    input_governance_cells: Vec<usize>,
    output_governance_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    verifiers::basic::verify_cell_number_and_position(
        "GovernanceMemberCell",
        &input_governance_cells,
        &[0],
        &output_governance_cells,
        &[0],
    )?;

    verify_the_role_of_target(
        input_governance_cells[0],
        GovernanceMemberRole::Custodian,
        "This transaction can only update the custodian members.".to_string(),
    )?;

    // Only data field can be updated
    verifiers::basic::verify_cell_consistent_with_exception(
        "GovernanceMemberCell",
        input_governance_cells[0],
        output_governance_cells[0],
        vec![CellField::Data, CellField::Capacity],
    )?;

    verify_the_custodian_cell_data(output_governance_cells[0])?;

    Ok(())
}

fn update_merchants(
    input_governance_cells: Vec<usize>,
    output_governance_cells: Vec<usize>,
) -> Result<(), Box<dyn AsI8>> {
    let custodian_lock = verifiers::permission::verify_input_has_custodian_lock(1)?;

    verifiers::basic::verify_cell_number_and_position(
        "GovernanceMemberCell",
        &input_governance_cells,
        &[0],
        &output_governance_cells,
        &[0],
    )?;

    verify_the_role_of_target(
        input_governance_cells[0],
        GovernanceMemberRole::Merchant,
        "This transaction can only update the merchant members.".to_string(),
    )?;

    // Only data field can be updated
    verifiers::basic::verify_cell_consistent_with_exception(
        "GovernanceMemberCell",
        input_governance_cells[0],
        output_governance_cells[0],
        vec![CellField::Data],
    )?;

    verify_the_merchant_cell_data(output_governance_cells[0])?;
    verify_the_custodian_not_in_merchants(&custodian_lock, output_governance_cells[0])?;

    Ok(())
}

fn verify_cell_id_correct(cell_id: &[u8], output_index: usize) -> Result<(), GovernanceError> {
    debug!("Verify if the cell ID is correct.");

    let input_0 = high_level::load_input(0, Source::Input).map_err(GovernanceError::from)?;
    let type_id = util::build_type_id(&input_0, output_index as u64);

    cc_assert!(
        &cell_id == &type_id,
        GovernanceError::CellIdIsInvalid {
            current: hex::encode(&cell_id),
            expected: hex::encode(&type_id)
        }
    );

    Ok(())
}

fn verify_the_role_of_target(
    index: usize,
    expected_role: GovernanceMemberRole,
    msg: String,
) -> Result<(), Box<dyn AsI8>> {
    let (role, _cell_id) = util::load_governance_member_type_info(index, Source::Output)?;

    cc_assert!(
        role == expected_role,
        GovernanceError::PermissionDenied { msg: msg.clone() }
    );

    Ok(())
}

fn verify_the_custodian_cell_lock(index: usize) -> Result<(), GovernanceError> {
    debug!("Verify if the GovernanceMemberCell in outputs has owner lock.");

    let lock = high_level::load_cell_lock(index, Source::Output).map_err(GovernanceError::from)?;
    let owner_lock = owner_lock();
    cc_assert!(
        util::is_entity_eq(owner_lock, &lock),
        GovernanceError::NewCellLockError {
            current: lock.to_string(),
            expected: owner_lock.to_string()
        }
    );

    Ok(())
}

fn verify_the_custodian_cell_data(index: usize) -> Result<(), Box<dyn AsI8>> {
    debug!("Verify if the GovernanceMemberCell.data is valid.");

    let data = high_level::load_cell_data(index, Source::Output).map_err(GovernanceError::from)?;
    let (version, governance_members) = governance_member_cell::parse_data(&data)?;

    match version {
        0 => {
            cc_assert!(
                governance_members.parent_id().is_empty(),
                GovernanceError::CustodianParentIdMustBeEmpty
            );

            let current_multisig_args = governance_members.multisig_args().as_reader().raw_data().to_vec();

            cc_assert!(
                current_multisig_args.len() == 3,
                GovernanceError::CustodianMultiSigArgsIsInvalid
            );

            let mut pubkey_hashes = vec![];
            for member in governance_members.members().into_iter() {
                pubkey_hashes.push(member.as_reader().raw_data().to_vec());
            }

            let require_first_n = current_multisig_args[1];
            let threshold = current_multisig_args[2];
            let expected_lock_args = util::build_omni_lock_multisig_args(require_first_n, threshold, pubkey_hashes);
            let current_lock_args = governance_members.as_reader().lock_args().raw_data();

            cc_assert!(
                current_lock_args == &expected_lock_args,
                GovernanceError::CustodianLockArgsInDataIsInvalid {
                    expected: hex::encode(&expected_lock_args),
                    current: hex::encode(current_lock_args)
                }
            );
        }
        _ => return Err(GovernanceError::UnsupportedDataVersion { version }.into()),
    }

    Ok(())
}

fn verify_the_merchant_cell_data(index: usize) -> Result<(), Box<dyn AsI8>> {
    debug!("Verify if the GovernanceMemberCell.data is valid.");

    let data = high_level::load_cell_data(index, Source::Output).map_err(GovernanceError::from)?;
    let (version, governance_members) = governance_member_cell::parse_data(&data)?;
    match version {
        0 => {
            cc_assert!(
                !governance_members.parent_id().is_empty(),
                GovernanceError::MerchantParentIdMustNotBeEmpty
            );

            cc_assert!(
                governance_members.lock_args().is_empty(),
                GovernanceError::MerchantLockArgsMustBeEmpty
            );

            cc_assert!(
                governance_members.multisig_args().is_empty(),
                GovernanceError::MerchantMultisigArgsMustBeEmpty
            );

            let custodian_cell_index = verifiers::permission::verify_and_find_governance_cell(
                GovernanceMemberRole::Custodian,
                Source::CellDep,
            )?;
            let (_role, expected_parent_id) =
                util::load_governance_member_type_info(custodian_cell_index, Source::CellDep)?;

            let parent_id = governance_members.as_reader().parent_id().raw_data();
            cc_assert!(
                parent_id == &expected_parent_id,
                GovernanceError::MerchantParentIdMismatch {
                    current: hex::encode(parent_id),
                    expected: hex::encode(&expected_parent_id)
                }
            );
        }
        _ => return Err(GovernanceError::UnsupportedDataVersion { version }.into()),
    }

    Ok(())
}

fn verify_the_custodian_not_in_merchants(custodian_lock: &Script, index: usize) -> Result<(), Box<dyn AsI8>> {
    debug!("Verify if the custodian lock exists in GovernanceMemberCell.data.members .");

    let data = high_level::load_cell_data(index, Source::Output).map_err(GovernanceError::from)?;
    let (version, governance_members) = governance_member_cell::parse_data(&data)?;
    match version {
        0 => {
            let custodian_lock_slice = custodian_lock.as_reader().as_slice();
            for member in governance_members.members() {
                debug!("member: {}", hex::encode(member.as_slice()));
                cc_assert!(
                    member.raw_data().as_ref() != custodian_lock_slice,
                    GovernanceError::CustodianLockMustNotInMerchants
                );
            }
        }
        _ => return Err(GovernanceError::UnsupportedDataVersion { version }.into()),
    }

    Ok(())
}
