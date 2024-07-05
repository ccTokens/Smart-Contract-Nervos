use alloc::boxed::Box;

use ckb_std::error::SysError;
use contract_core::error::AsI8;
use thiserror_no_std::Error;

#[derive(Error, Debug)]
#[repr(i8)]
pub enum ConfigError {
    #[error("index out of bound")]
    IndexOutOfBound = 1,
    #[error("item missing")]
    ItemMissing,
    #[error("length not enough, the real length is {0}")]
    LengthNotEnough(usize),
    #[error("encoding error")]
    Encoding,
    // Custom errors
    #[error("The ConfigCell.type.args must be empty.")]
    ArgsMustBeEmpty,
}

impl From<SysError> for ConfigError {
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

impl AsI8 for ConfigError {
    fn as_i8(&self) -> i8 {
        match self {
            ConfigError::IndexOutOfBound => 1,
            ConfigError::ItemMissing => 2,
            ConfigError::LengthNotEnough(_) => 3,
            ConfigError::Encoding => 4,
            ConfigError::ArgsMustBeEmpty => 5,
        }
    }
}

impl From<ConfigError> for Box<dyn AsI8> {
    fn from(err: ConfigError) -> Self {
        Box::new(err)
    }
}
