extern crate serde_idl;
extern crate serde;

use serde::Serialize;
use serde_idl::{to_vec};

#[test]
fn test_bool() {
    check(true, "4449444c7e01");
    check(false, "4449444c7e00");
}

#[test]
fn test_integer() {
    check(42, "4449444c7c2a");
    check(1234567890, "4449444c7cd285d8cc04");
    check(-1234567890, "4449444c7caefaa7b37b");
 }

fn check<T: Serialize>(value: T, expected: &str) {
    let encoded = to_vec(&value).unwrap();
    let expected = hex::decode(expected).unwrap();
    assert_eq!(encoded, expected);
}
