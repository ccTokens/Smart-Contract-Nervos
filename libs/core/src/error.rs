use alloc::boxed::Box;
use alloc::string::String;

use ckb_std::ckb_constants::Source;
use ckb_std::error::SysError;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
#[repr(i8)]
pub enum CoreError {
    #[error("index out of bound")]
    IndexOutOfBound = 1,
    #[error("item missing")]
    ItemMissing,
    #[error("length not enough, the real length is {0}")]
    LengthNotEnough(usize),
    #[error("encoding error")]
    Encoding,
    #[error("There is something wrong with the data from .env file.")]
    DotEnvError,
    #[error("{msg}")]
    InvalidTransactionStructure { msg: String },
    #[error("witnesses[{index}] The action is not found.")]
    ActionNotFound { index: usize },
    #[error("witnesses[{index}] The action version {version} is undefined.")]
    ActionVersionUnknown { index: usize, version: u8 },
    #[error("witnesses[{index}] The action hex {hex} is undefined.")]
    ActionUndefined { index: usize, hex: String },
    #[error("The action {action} is not supported.")]
    ActionNotSupported { action: String },
    #[error("The version {version} is unsupported for {cell_name} .")]
    ParseCellDataVersionFailed { version: u8, cell_name: String },
    #[error("The {cell_name}.data is invalid: {msg}")]
    ParseCellDataFailed { cell_name: String, msg: String },
    #[error("The {cell_name}.type.args is invalid: {msg}")]
    ParseCellTypeArgsFailed { cell_name: String, msg: String },
    #[error("inputs[{index}] The cell must have deploy lock.")]
    DeployLockIsRequired { index: usize },
    #[error("inputs[{index}] The cell must have owner lock.")]
    OwnerLockIsRequired { index: usize },
    #[error("{source:?}[{index}] The cell must have always success lock.")]
    AlwaysSuccessLockIsRequired { index: usize, source: Source },
    #[error("inputs[{index}] The cell must have custodian lock.")]
    CustodianLockIsRequired { index: usize },
    #[error("The {cell_name}.lock must be owner lock.")]
    CellLockMustBeOwnerLock { cell_name: String },
    #[error("The {cell_name}.capacity must be consistent.")]
    CellCapacityMustBeConsistent { cell_name: String },
    #[error("The {cell_name}.lock can not be modified.")]
    CellLockMustBeConsistent { cell_name: String },
    #[error("The {cell_name}.type must be consistent.")]
    CellTypeMustBeConsistent { cell_name: String },
    #[error("The {cell_name}.data must be consistent.")]
    CellDataMustBeConsistent { cell_name: String },
    #[error("{source:?}[{index}] The GovernanceMemberCell.lock is invalid, custodian should use owner lock, merchant should use always_success.")]
    GovernanceCellLockMismatch { index: usize, source: Source },
    #[error("{source:?}[{index}] The GovernanceMemberCell is corrupted: {msg}")]
    GovernanceCellIsCorrupted { index: usize, source: Source, msg: String },
    #[error("{source:?}[{index}] The GovernanceMemberCell.type.args.role should be {expected}, but {current} found.")]
    GovernanceCellRoleError {
        index: usize,
        source: Source,
        expected: String,
        current: String,
    },
    #[error("inputs[{index}] The cell must have merchant lock.")]
    MerchantLockIsRequired { index: usize },
    #[error("The system status is off.")]
    SystemStatusOff,
    #[error("Parse length value field {field_name} failed.")]
    ParseLvFailed { field_name: String },
}

impl From<SysError> for CoreError {
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

pub trait AsI8 {
    fn as_i8(&self) -> i8;
}

impl AsI8 for CoreError {
    fn as_i8(&self) -> i8 {
        match self {
            CoreError::IndexOutOfBound => 1,
            CoreError::ItemMissing => 2,
            CoreError::LengthNotEnough(_) => 3,
            CoreError::Encoding => 4,
            CoreError::DotEnvError => 5,
            CoreError::InvalidTransactionStructure { msg: _ } => 6,
            CoreError::ActionNotFound { index: _ } => 7,
            CoreError::ActionVersionUnknown { index: _, version: _ } => 8,
            CoreError::ActionUndefined { index: _, hex: _ } => 9,
            CoreError::ActionNotSupported { action: _ } => 10,
            CoreError::ParseCellDataVersionFailed {
                version: _,
                cell_name: _,
            } => 11,
            CoreError::ParseCellDataFailed { cell_name: _, msg: _ } => 12,
            CoreError::ParseCellTypeArgsFailed { cell_name: _, msg: _ } => 13,
            CoreError::DeployLockIsRequired { index: _ } => 14,
            CoreError::OwnerLockIsRequired { index: _ } => 15,
            CoreError::AlwaysSuccessLockIsRequired { index: _, source: _ } => 16,
            CoreError::CustodianLockIsRequired { index: _ } => 17,
            CoreError::CellLockMustBeOwnerLock { cell_name: _ } => 18,
            CoreError::CellCapacityMustBeConsistent { cell_name: _ } => 19,
            CoreError::CellLockMustBeConsistent { cell_name: _ } => 20,
            CoreError::CellTypeMustBeConsistent { cell_name: _ } => 21,
            CoreError::CellDataMustBeConsistent { cell_name: _ } => 22,
            CoreError::GovernanceCellLockMismatch { index: _, source: _ } => 23,
            CoreError::GovernanceCellIsCorrupted {
                index: _,
                source: _,
                msg: _,
            } => 24,
            CoreError::GovernanceCellRoleError {
                index: _,
                source: _,
                expected: _,
                current: _,
            } => 25,
            CoreError::MerchantLockIsRequired { index: _ } => 26,
            CoreError::SystemStatusOff => 27,
            CoreError::ParseLvFailed { field_name: _ } => 28,
        }
    }
}

impl From<CoreError> for Box<dyn AsI8> {
    fn from(err: CoreError) -> Self {
        Box::new(err)
    }
}
