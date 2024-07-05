use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::ToString;

use ckb_std::ckb_constants::Source;
use ckb_std::ckb_types::core::ScriptHashType;
use ckb_std::high_level;
use ckb_std::high_level::load_cell_lock;
use types::constants::{deploy_lock, owner_lock, GovernanceMemberRole};
use types::packed::{Byte32, GovernanceMembers, Reader, Script};
use types::prelude::{Builder, Entity};
use types::util::hex_string;

use crate::config::{always_success_lock, governance_member_cell_type_id, omni_lock_type_id};
use crate::constants::ScriptType;
use crate::data_parser::governance_member_cell;
use crate::error::CoreError;
use crate::util;

pub fn verify_input_has_deploy_lock(index: usize) -> Result<(), CoreError> {
    let deploy_lock = deploy_lock();
    let cells = util::find_cells_by_script(ScriptType::Lock, deploy_lock.as_reader(), Source::Input)?;

    cc_assert!(!cells.is_empty(), CoreError::DeployLockIsRequired { index });
    cc_assert!(cells[0] == index, CoreError::DeployLockIsRequired { index });

    Ok(())
}

pub fn verify_input_has_owner_lock(index: usize) -> Result<(), CoreError> {
    let owner_lock = owner_lock();
    let cells = util::find_cells_by_script(ScriptType::Lock, owner_lock.as_reader(), Source::Input)?;

    cc_assert!(!cells.is_empty(), CoreError::OwnerLockIsRequired { index });
    cc_assert!(cells[0] == index, CoreError::OwnerLockIsRequired { index });

    Ok(())
}

pub fn verify_governance_cell_role(
    expected_role: GovernanceMemberRole,
    index: usize,
    source: Source,
) -> Result<(), CoreError> {
    debug!(
        "{:?}[{}] Verify if the GovernanceMemberCell has {:?} .",
        source,
        index,
        expected_role.to_string()
    );

    let type_args = match high_level::load_cell_type(index, source)? {
        Some(type_) => type_.args().as_reader().raw_data().to_vec(),
        None => {
            return Err(CoreError::GovernanceCellIsCorrupted {
                index,
                source,
                msg: format!("the type.args must not be empty"),
            })
        }
    };

    let (role, _) = governance_member_cell::parse_type_args(&type_args)?;
    cc_assert!(
        role == expected_role,
        CoreError::GovernanceCellRoleError {
            index,
            source,
            expected: expected_role.to_string(),
            current: role.to_string()
        }
    );

    Ok(())
}

pub fn verify_and_find_governance_cell(role: GovernanceMemberRole, source: Source) -> Result<usize, CoreError> {
    let type_id = Byte32::try_from(governance_member_cell_type_id()?).unwrap();
    debug!(
        "Find GovernanceMemberCell by type_id: {}",
        hex_string(type_id.as_reader().as_slice())
    );
    let cell_index =
        util::find_only_cell_by_type_id("GovernanceMemberCell", ScriptType::Type, type_id.as_reader(), source)?;

    let expected_lock = match role {
        GovernanceMemberRole::Custodian => owner_lock().to_owned(),
        GovernanceMemberRole::Merchant => always_success_lock()?,
    };
    let lock = high_level::load_cell_lock(cell_index, source)?;
    cc_assert!(
        util::is_entity_eq(&expected_lock, &lock),
        CoreError::GovernanceCellLockMismatch {
            index: cell_index,
            source
        }
    );

    verify_governance_cell_role(role, cell_index, source)?;

    Ok(cell_index)
}

pub fn verify_input_has_custodian_lock(index: usize) -> Result<Script, CoreError> {
    debug!("inputs[{}] Verify if the cell has custodian lock.", index);

    let custodian_cell_index = verify_and_find_governance_cell(GovernanceMemberRole::Custodian, Source::CellDep)?;

    let data = high_level::load_cell_data(custodian_cell_index, Source::CellDep).map_err(CoreError::from)?;
    let (_version, members) = governance_member_cell::parse_data(&data)?;

    let custodian_lock = build_custodian_lock(&members)?;
    let cells = util::find_cells_by_script(ScriptType::Lock, custodian_lock.as_reader(), Source::Input)?;

    debug!("Expected custodian lock: {}", custodian_lock);

    cc_assert!(!cells.is_empty(), CoreError::CustodianLockIsRequired { index });
    cc_assert!(cells[0] == index, CoreError::CustodianLockIsRequired { index });

    Ok(custodian_lock)
}

pub fn verify_input_has_merchant_lock(index: usize) -> Result<(), CoreError> {
    debug!("inputs[{}] Verify if the cell has merchant lock.", index);

    let merchant_cell_index = verify_and_find_governance_cell(GovernanceMemberRole::Merchant, Source::CellDep)?;

    let data = high_level::load_cell_data(merchant_cell_index, Source::CellDep).map_err(CoreError::from)?;
    let (_version, members) = governance_member_cell::parse_data(&data)?;

    let input_lock = load_cell_lock(index, Source::Input)?;
    let input_lock_slice = input_lock.as_slice();
    let mut is_merchant = false;
    for member in members.members() {
        debug!("member: {}", hex::encode(member.as_slice()));
        if member.raw_data().as_ref() == input_lock_slice {
            is_merchant = true;
            break;
        }
    }
    cc_assert!(is_merchant, CoreError::MerchantLockIsRequired { index });
    Ok(())
}

pub fn verify_cell_has_always_success_lock(index: usize, source: Source) -> Result<(), CoreError> {
    debug!("{:?}[{}] Verify if the cell has always_success lock.", source, index);

    let always_success_lock = always_success_lock()?;
    let lock = high_level::load_cell_lock(index, source)?;

    cc_assert!(
        util::is_entity_eq(&always_success_lock, &lock),
        CoreError::AlwaysSuccessLockIsRequired { index, source }
    );

    Ok(())
}

fn build_custodian_lock(members: &GovernanceMembers) -> Result<Script, CoreError> {
    let code_hash = Byte32::try_from(omni_lock_type_id()?).unwrap();
    let args = members.lock_args();

    Ok(Script::new_builder()
        .code_hash(code_hash)
        .hash_type(ScriptHashType::Type.into())
        .args(args)
        .build())
}
