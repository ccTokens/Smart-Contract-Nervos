use alloc::boxed::Box;
use alloc::string::String;

use ckb_std::error::SysError;
use contract_core::error::AsI8;
use thiserror_no_std::Error;

/// Error
#[derive(Error, Debug)]
#[repr(i8)]
pub enum TickError {
    #[error("index out of bound")]
    IndexOutOfBound = 1,
    #[error("item missing")]
    ItemMissing,
    #[error("length not enough")]
    LengthNotEnough,
    #[error("encoding error")]
    Encoding,
    // Add customized errors here...
    #[error("The TickCell.data.type {0} is not supported.")]
    UnsupportedTickType(u8),
    #[error("The TickCell.data.value can not be 0 .")]
    TickValueCanNotBeZero,
    #[error("The TickCell.data.type is invalid.(current: {current}, expected: {expected})")]
    InvalidTickType { current: String, expected: String },
    #[error("The size of TickCell.data.token_id is invalid.")]
    InvalidTickTokenIdSize,
    #[error("Tick cell tick.merchant illegal, the merchant lock is {lock}")]
    InvalidTickMerchantLock { lock: String },
    #[error("The XudtCell.type.args.token_id does not match the TickCell.(expected: {expected}, current: {current})")]
    XudtCellTokenIdMismatch { current: String, expected: String },
    #[error("{source}[{index}] XudtCell.type.args decoding failed")]
    UnsupportedXudtTypeArgs { index: usize, source: String },
    #[error("{source}[{index}] XudtCell.data should always start with a u128 in LE .")]
    UnsupportedXudtData { index: usize, source: String },
    #[error("{source}[{index}] The extension script in XudtCell.lock.args is unknown.")]
    UnsupportedXudtExtensionScript { index: usize, source: String },
    #[error("{source}[{index}] Found multiple kind of XudtCell, but only one kind is supported.")]
    MultipleKindOfXudtFound { index: usize, source: String },
    #[error("The burned Xudt amount {burned} does not match the expected amount {expected} in TickCell")]
    BurnedXudtAmountNotMatch { burned: u128, expected: u128 },
    #[error("There should be {amount} {token_id} token transferred to {target_lock} .")]
    XudtTransferError {
        target_lock: String,
        token_id: String,
        amount: u128,
    },
}

impl From<SysError> for TickError {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}

impl AsI8 for TickError {
    fn as_i8(&self) -> i8 {
        match self {
            TickError::IndexOutOfBound => 1,
            TickError::ItemMissing => 2,
            TickError::LengthNotEnough => 3,
            TickError::Encoding => 4,
            TickError::UnsupportedTickType(_) => 5,
            TickError::TickValueCanNotBeZero => 6,
            TickError::InvalidTickType { .. } => 7,
            TickError::InvalidTickTokenIdSize => 8,
            TickError::InvalidTickMerchantLock { .. } => 9,
            TickError::XudtCellTokenIdMismatch { .. } => 10,
            TickError::UnsupportedXudtTypeArgs { .. } => 11,
            TickError::UnsupportedXudtData { .. } => 12,
            TickError::UnsupportedXudtExtensionScript { .. } => 13,
            TickError::MultipleKindOfXudtFound { .. } => 14,
            TickError::BurnedXudtAmountNotMatch { .. } => 15,
            TickError::XudtTransferError { .. } => 16,
        }
    }
}

impl From<TickError> for Box<dyn AsI8> {
    fn from(err: TickError) -> Self {
        Box::new(err)
    }
}
