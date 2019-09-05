extern crate serde_idl;
extern crate serde;
extern crate dfx_info;

use serde::Serialize;
use serde_idl::{to_vec};
use dfx_info::{DfinityInfo, Type, get_type};

#[test]
fn test_bool() {
    check(true, "4449444c007e01");
    check(false, "4449444c007e00");
    assert_eq!(get_type(&true), Type::Bool);
}

#[test]
fn test_integer() {
    check(42, "4449444c007c2a");
    check(1234567890, "4449444c007cd285d8cc04");
    check(-1234567890, "4449444c007caefaa7b37b");
    check(Box::new(42), "4449444c007c2a");
    assert_eq!(get_type(&42), Type::Int);
}

#[test]
fn test_option() {
    check(Some(42), "4449444c016e7c00012a");
    check(Some(Some(42)), "4449444c026e7c6e000101012a");
    let opt: Option<i32> = None;
    assert_eq!(get_type(&opt), Type::Opt(Box::new(Type::Int)));
    //check(opt, "4449444c");
}

#[derive(Serialize, Debug, DfinityInfo)]
struct A { foo: i32, bar: bool }
#[derive(Serialize, Debug, DfinityInfo)]
struct List { head: i32, tail: Option<Box<List>> }
#[derive(Serialize, Debug, DfinityInfo)]
enum E { Foo, Bar(bool), Baz{a: i32, b: u32} }


fn field(id: &str, ty: Type) -> dfx_info::Field {
    dfx_info::Field { id: id.to_string(), ty: ty }
}

#[test]
fn test_struct() {
    let record = A { foo: 42, bar: true };
    check(record, "4449444c016c02d3e3aa027e868eb7027c00012a");
    let record = A { foo: 42, bar: true };    
    assert_eq!(get_type(&record),
               Type::Record(vec![
                   field("foo", Type::Int),
                   field("bar", Type::Bool)]));
    //let list = List { head: 42, tail: None };
    //assert_eq!(get_type(&list), Type::Bool);
    //check(List { head: 42, tail: None }, "4449444c016c02d3");
}

#[test]
fn test_variant() {
    let v = E::Foo;
    assert_eq!(get_type(&v),
               Type::Variant(vec![
                   field("Foo", Type::Null),
                   field("Bar", Type::Record(vec![field("0", Type::Bool)])),
                   field("Baz", Type::Record(vec![field("a", Type::Int),
                                                  field("b", Type::Nat)])),
                   ])
    );
}

fn check<T: Serialize>(value: T, expected: &str) {
    let encoded = to_vec(&value).unwrap();
    let expected = hex::decode(expected).unwrap();
    assert_eq!(encoded, expected, "\nExpected\n{:x?}\nActual\n{:x?}\n", expected, encoded);
}
