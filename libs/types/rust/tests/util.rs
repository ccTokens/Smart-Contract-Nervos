use core::convert::TryFrom;
use std::convert::TryInto;

use das_types::constants::{DataType, Source, WITNESS_HEADER};
use das_types::packed::*;
use das_types::prelude::*;
use das_types::util::{self, EntityWrapper};
use hex;

#[test]
fn test_is_entity_eq() {
    let a = Bytes::from("aaa".as_bytes());
    let b = Bytes::from("aaa".as_bytes());
    assert!(
        util::is_entity_eq(&a, &b),
        "Function is_entity_eq should return true if bytes are the same."
    );

    let a = Bytes::from("aaa".as_bytes());
    let b = Bytes::from("bbb".as_bytes());
    assert!(
        !util::is_entity_eq(&a, &b),
        "Function is_entity_eq should return false if bytes are not the same."
    );
}

#[test]
fn test_is_reader_eq() {
    let a = Bytes::from("aaa".as_bytes());
    let b = Bytes::from("aaa".as_bytes());
    assert!(
        util::is_reader_eq(a.as_reader(), b.as_reader()),
        "Function is_reader_eq should return true if bytes are the same."
    );

    let a = Bytes::from("aaa".as_bytes());
    let b = Bytes::from("bbb".as_bytes());
    assert!(
        !util::is_reader_eq(a.as_reader(), b.as_reader()),
        "Function is_reader_eq should return false if bytes are not the same."
    );
}
