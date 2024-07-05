#[cfg(feature = "no_std")]
use blake2b_ref::Blake2b;
#[cfg(feature = "no_std")]
use blake2b_ref::Blake2bBuilder;
#[cfg(not(feature = "no_std"))]
use blake2b_rs::Blake2b;
#[cfg(not(feature = "no_std"))]
use blake2b_rs::Blake2bBuilder;
pub use molecule::hex_string;
use molecule::prelude::*;

use super::constants::*;

pub fn is_entity_eq<T: Entity>(a: &T, b: &T) -> bool {
    a.as_slice() == b.as_slice()
}

pub fn is_reader_eq<'a, T: Reader<'a>>(a: T, b: T) -> bool {
    a.as_slice() == b.as_slice()
}

pub fn blake2b_256<T: AsRef<[u8]>>(s: T) -> [u8; 32] {
    let mut result = [0u8; CKB_HASH_DIGEST];
    let mut blake2b = Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build();
    blake2b.update(s.as_ref());
    blake2b.finalize(&mut result);
    result
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(CKB_HASH_DIGEST)
        .personal(CKB_HASH_PERSONALIZATION)
        .build()
}
