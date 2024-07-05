use alloc::boxed::Box;
use alloc::string::String;

use ckb_std::error::SysError;
use contract_core::error::AsI8;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
#[repr(i8)]
pub enum GovernanceError {
    #[error("index out of bound")]
    IndexOutOfBound = 1,
    #[error("item missing")]
    ItemMissing,
    #[error("length not enough, the real length is {0}")]
    LengthNotEnough(usize),
    #[error("encoding error")]
    Encoding,
    // Custom errors
    #[error("outputs[0] The cell.type.args.cell_id is invalid.(expected: {expected}, current: {current})")]
    CellIdIsInvalid { current: String, expected: String },
    #[error("outputs[0] The cell.lock is invalid.(expected: {expected}, current: {current})")]
    NewCellLockError { current: String, expected: String },
    #[error("The GovernanceMemberCell.data.version {version} is not supported.")]
    UnsupportedDataVersion { version: u8 },
    #[error("The GovernanceMemberCell(custodian).data.parent_id must be empty.")]
    CustodianParentIdMustBeEmpty,
    #[error("The GovernanceMemberCell(merchant).data.parent_id must not be empty.")]
    MerchantParentIdMustNotBeEmpty,
    #[error("The GovernanceMemberCell(merchant).data.lock_args must be empty.")]
    MerchantLockArgsMustBeEmpty,
    #[error("The GovernanceMemberCell(merchant).data.multisig_args must be empty.")]
    MerchantMultisigArgsMustBeEmpty,
    #[error("The GovernanceMemberCell(custodian) is required in cell_deps.")]
    CustodianCellIsRequired,
    #[error("The GovernanceMemberCell(merchant).data.parent_id is invalid.(current: {current}, expected: {expected})")]
    MerchantParentIdMismatch { current: String, expected: String },
    #[error("The GovernanceMemberCell(custodian).data.multisig_args should be 3 bytes.")]
    CustodianMultiSigArgsIsInvalid,
    #[error(
        "The GovernanceMemberCell(custodian).data.lock_args is invalid.(current: {current}, expected: {expected})"
    )]
    CustodianLockArgsInDataIsInvalid { current: String, expected: String },
    #[error("Can not send the transaction: {msg}")]
    PermissionDenied { msg: String },
    #[error("The omni-lock of custodians must not exsit in GovernanceMemberCell(merchants).data.members .")]
    CustodianLockMustNotInMerchants,
}

impl From<SysError> for GovernanceError {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(len) => Self::LengthNotEnough(len),
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl AsI8 for GovernanceError {
    fn as_i8(&self) -> i8 {
        match self {
            GovernanceError::IndexOutOfBound => 1,
            GovernanceError::ItemMissing => 2,
            GovernanceError::LengthNotEnough(_) => 3,
            GovernanceError::Encoding => 4,
            GovernanceError::CellIdIsInvalid {
                current: _,
                expected: _,
            } => 5,
            GovernanceError::NewCellLockError {
                current: _,
                expected: _,
            } => 6,
            GovernanceError::UnsupportedDataVersion { version: _ } => 7,
            GovernanceError::CustodianParentIdMustBeEmpty => 8,
            GovernanceError::MerchantParentIdMustNotBeEmpty => 9,
            GovernanceError::MerchantLockArgsMustBeEmpty => 10,
            GovernanceError::MerchantMultisigArgsMustBeEmpty => 11,
            GovernanceError::CustodianCellIsRequired => 12,
            GovernanceError::MerchantParentIdMismatch {
                current: _,
                expected: _,
            } => 13,
            GovernanceError::CustodianMultiSigArgsIsInvalid => 14,
            GovernanceError::CustodianLockArgsInDataIsInvalid {
                current: _,
                expected: _,
            } => 15,
            GovernanceError::PermissionDenied { msg: _ } => 16,
            GovernanceError::CustodianLockMustNotInMerchants => 17,
        }
    }
}

impl From<GovernanceError> for Box<dyn AsI8> {
    fn from(err: GovernanceError) -> Self {
        Box::new(err)
    }
}
