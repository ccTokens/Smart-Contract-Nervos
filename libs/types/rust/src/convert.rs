#[cfg(feature = "no_std")]
use alloc::borrow::ToOwned;
#[cfg(feature = "no_std")]
use alloc::string::{FromUtf8Error, String};
#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(feature = "no_std")]
use core::convert::TryFrom;
#[cfg(not(feature = "no_std"))]
use std::convert::TryFrom;
#[cfg(not(feature = "no_std"))]
use std::string::FromUtf8Error;

#[cfg(feature = "no_std")]
use ckb_std::ckb_types::{bytes, packed as ckb_packed};
#[cfg(not(feature = "no_std"))]
use ckb_types::{bytes, packed as ckb_packed};
// #[cfg(not(feature = "no_std"))]
// use ckb_types::prelude::{Reader, Entity};
// #[cfg(feature = "no_std")]
use molecule::error::VerificationError;
use molecule::prelude::{Builder, Entity, Reader};

use super::schemas::packed::*;

/// Implement convert between primitive type and molecule types
macro_rules! impl_uint_convert {
    ($uint_type:ty, $mol_type:ident, $reader_type:ident, $length: expr, $flag: tt) => {
        impl From<ckb_packed::$mol_type> for $mol_type {
            fn from(v: ckb_packed::$mol_type) -> Self {
                Self::new_unchecked(v.as_bytes().into())
            }
        }

        impl<'r> From<ckb_packed::$reader_type<'r>> for $reader_type<'r> {
            fn from(v: ckb_packed::$reader_type<'r>) -> $reader_type<'r> {
                $reader_type::new_unchecked(v.as_slice())
            }
        }

        impl<'r> Into<ckb_packed::$reader_type<'r>> for $reader_type<'r> {
            fn into(self) -> ckb_packed::$reader_type<'r> {
                ckb_packed::$reader_type::new_unchecked(self.as_slice())
            }
        }

        impl_uint_convert!($uint_type, $mol_type, $reader_type, $length);
    };
    ($uint_type:ty, $mol_type:ident, $reader_type:ident, $length: expr) => {
        impl From<$uint_type> for $mol_type {
            fn from(v: $uint_type) -> Self {
                Self::new_unchecked(bytes::Bytes::from(v.to_le_bytes().to_vec()))
            }
        }

        impl From<$mol_type> for $uint_type {
            fn from(v: $mol_type) -> Self {
                let mut buf = [0u8; $length];
                buf.copy_from_slice(v.raw_data().as_ref());
                <$uint_type>::from_le_bytes(buf)
            }
        }

        impl From<$reader_type<'_>> for $uint_type {
            fn from(v: $reader_type<'_>) -> Self {
                let mut buf = [0u8; $length];
                buf.copy_from_slice(v.raw_data());
                <$uint_type>::from_le_bytes(buf)
            }
        }
    };
}

impl_uint_convert!(u8, Uint8, Uint8Reader, 1);
impl_uint_convert!(u32, Uint32, Uint32Reader, 4, compatible);
impl_uint_convert!(u64, Uint64, Uint64Reader, 8, compatible);
impl_uint_convert!(u128, Uint128, Uint128Reader, 16, compatible);

/// Convert &[u8] to schemas::basic::Bytes
///
/// The difference with from_slice is that it does not require a dynvec header.
impl From<&[u8]> for Bytes {
    fn from(v: &[u8]) -> Self {
        Bytes::new_builder()
            .set(v.to_owned().into_iter().map(Byte::new).collect())
            .build()
    }
}

/// Convert Vec<u8> to schemas::basic::Bytes
///
/// The difference with from_slice is that it does not require a dynvec header.
impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Bytes::from(v.as_slice())
    }
}

/// Convert bytes::Bytes to schemas::basic::Bytes
impl From<bytes::Bytes> for Bytes {
    fn from(v: bytes::Bytes) -> Self {
        Bytes::from(v.as_ref())
    }
}

impl From<ckb_packed::Bytes> for Bytes {
    fn from(v: ckb_packed::Bytes) -> Self {
        Bytes::new_unchecked(v.as_bytes().into())
    }
}

impl Into<ckb_packed::Bytes> for Bytes {
    fn into(self) -> ckb_packed::Bytes {
        ckb_packed::Bytes::new_unchecked(self.as_bytes().into())
    }
}

impl<'r> From<ckb_packed::BytesReader<'r>> for BytesReader<'r> {
    fn from(v: ckb_packed::BytesReader<'r>) -> Self {
        BytesReader::new_unchecked(v.as_slice())
    }
}

impl<'r> Into<ckb_packed::BytesReader<'r>> for BytesReader<'r> {
    fn into(self) -> ckb_packed::BytesReader<'r> {
        ckb_packed::BytesReader::new_unchecked(self.as_slice())
    }
}

/// Convert schemas::basic::Bytes to Vec<u8>
///
/// The main thing here is to remove the Header from the Molecule data.
impl From<Bytes> for Vec<u8> {
    fn from(v: Bytes) -> Self {
        v.as_reader().raw_data().to_vec()
    }
}

impl From<Option<Vec<u8>>> for BytesOpt {
    fn from(v: Option<Vec<u8>>) -> Self {
        BytesOpt::new_builder().set(v.map(Bytes::from)).build()
    }
}

impl Into<Option<Vec<u8>>> for BytesOpt {
    fn into(self) -> Option<Vec<u8>> {
        self.to_opt().map(|v| v.into())
    }
}

/// Convert schemas::basic::Bytes to String
///
/// The main thing here is to remove the Header from the Molecule data.
impl TryFrom<Bytes> for String {
    type Error = FromUtf8Error;
    fn try_from(v: Bytes) -> Result<Self, FromUtf8Error> {
        let bytes = v.as_reader().raw_data().to_vec();
        String::from_utf8(bytes).map(|v| String::from(v))
    }
}

/// Convert &[u8] to schemas::basic::Byte32
///
/// The difference with from_slice is that it does not require a dynvec header.
impl TryFrom<&[u8]> for Byte32 {
    type Error = VerificationError;
    fn try_from(v: &[u8]) -> Result<Self, VerificationError> {
        if v.len() != 32 {
            return Err(VerificationError::TotalSizeNotMatch("Byte32".to_owned(), 32, v.len()));
        }
        let mut inner = [Byte::new(0); 32];
        let v = v.to_owned().into_iter().map(Byte::new).collect::<Vec<_>>();
        inner.copy_from_slice(&v);
        Ok(Self::new_builder().set(inner).build())
    }
}

impl TryFrom<Vec<u8>> for Byte32 {
    type Error = VerificationError;
    fn try_from(v: Vec<u8>) -> Result<Self, VerificationError> {
        Byte32::try_from(v.as_slice())
    }
}

/// Convert schemas::basic::Byte32 to Vec<u8>
impl From<Byte32> for Vec<u8> {
    fn from(v: Byte32) -> Self {
        v.as_slice().to_vec()
    }
}

impl From<[u8; 32]> for Byte32 {
    fn from(v: [u8; 32]) -> Self {
        let mut inner = [Byte::new(0); 32];
        for (i, item) in v.iter().enumerate() {
            inner[i] = Byte::new(*item);
        }
        Self::new_builder().set(inner).build()
    }
}

impl Into<[u8; 32]> for Byte32 {
    fn into(self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(self.as_slice());
        buf
    }
}

impl Into<[u8; 32]> for Byte32Reader<'_> {
    fn into(self) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(self.as_slice());
        buf
    }
}

/// Convert between schemas::Byte32 and ckb_types::ckb_packed::Byte32
impl From<ckb_packed::Byte32> for Byte32 {
    fn from(v: ckb_packed::Byte32) -> Self {
        Byte32::new_unchecked(v.as_bytes().into())
    }
}

impl<'r> From<ckb_packed::Byte32Reader<'r>> for Byte32Reader<'r> {
    fn from(v: ckb_packed::Byte32Reader<'r>) -> Self {
        Byte32Reader::new_unchecked(v.as_slice())
    }
}

impl Into<ckb_packed::Byte32> for Byte32 {
    fn into(self) -> ckb_packed::Byte32 {
        ckb_packed::Byte32::new_unchecked(self.as_bytes().into())
    }
}

impl<'r> Into<ckb_packed::Byte32Reader<'r>> for Byte32Reader<'r> {
    fn into(self) -> ckb_packed::Byte32Reader<'r> {
        ckb_packed::Byte32Reader::new_unchecked(self.as_slice())
    }
}

impl From<&str> for Byte32 {
    fn from(v: &str) -> Self {
        let hex = v.trim_start_matches("0x");
        let bytes: Vec<u8> = hex::decode(hex).expect("Expect input to valid hex");
        Byte32::from_compatible_slice(&bytes).expect("Convert hex to Byte32 should not fail")
    }
}

impl From<ckb_packed::Script> for Script {
    fn from(v: ckb_packed::Script) -> Self {
        Script::new_unchecked(v.as_bytes().into())
    }
}

impl<'r> From<ckb_packed::ScriptReader<'r>> for ScriptReader<'r> {
    fn from(v: ckb_packed::ScriptReader<'r>) -> Self {
        ScriptReader::new_unchecked(v.as_slice())
    }
}

impl Into<ckb_packed::Script> for Script {
    fn into(self) -> ckb_packed::Script {
        ckb_packed::Script::new_unchecked(self.as_bytes().into())
    }
}

impl<'r> Into<ckb_packed::ScriptReader<'r>> for ScriptReader<'r> {
    fn into(self) -> ckb_packed::ScriptReader<'r> {
        ckb_packed::ScriptReader::new_unchecked(self.as_slice())
    }
}

impl From<Option<Script>> for ScriptOpt {
    fn from(v: Option<Script>) -> Self {
        ScriptOpt::new_builder().set(v).build()
    }
}

impl Into<Option<Script>> for ScriptOpt {
    fn into(self) -> Option<Script> {
        self.to_opt()
    }
}
