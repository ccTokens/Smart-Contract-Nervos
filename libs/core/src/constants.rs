pub use ckb_std::ckb_types::core::ScriptHashType;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ScriptType {
    Lock,
    Type,
}

#[derive(Debug)]
pub enum LockScript {
    AlwaysSuccessLock,
    DasLock,
    Secp256k1Blake160SignhashLock,
    Secp256k1Blake160MultisigLock,
}

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum CellField {
    Capacity,
    Lock,
    Type,
    Data,
}

pub const CKB_HASH_DIGEST: usize = 32;
pub const CKB_HASH_PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub const ONE_CKB: u64 = 100_000_000;
pub const CELL_BASIC_CAPACITY: u64 = 6_1 * ONE_CKB;
pub const ONE_USD: u64 = 1_000_000;

pub const LV_HEADER_LENGTH: usize = 4;
pub const SECP_SIGNATURE_SIZE: usize = 65;

pub const TYPE_ID_CODE_HASH: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 84, 89, 80, 69, 95, 73, 68,
];
