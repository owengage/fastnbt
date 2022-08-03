use std::{collections::HashMap, iter::FromIterator};

use crate::{error::Result, from_bytes, test::builder::Builder, Tag, Value};

/// Bugs found via cargo-fuzz.

#[test]
fn partial_input_causes_panic_if_in_string() {
    let input = Builder::new().start_compound("some long name").build();
    let v: Result<Value> = from_bytes(&input[0..3]);
    assert!(v.is_err());
}

#[test]
fn list_of_end() {
    let input = Builder::new()
        .start_compound("")
        .start_list("", Tag::End, 1)
        .tag(Tag::End)
        .end_compound()
        .build();

    let v: Result<Value> = from_bytes(&input);
    assert!(v.is_err());
}

#[test]
fn float_double() {
    //           C   name  f  name  ............ end compound
    let input = [10, 0, 0, 5, 0, 0, 0, 0, 0, 10, 0];
    let v: Value = from_bytes(&input).unwrap();
    let expected = Value::Compound(HashMap::from_iter([(
        "".to_string(),
        Value::Float(1.4e-44),
    )]));

    assert_eq!(expected, v);
}

#[test]
fn overflow_len() {
    // This was a overflow caused by a multiply.
    let data = &[
        10, 0, 0, 12, 0, 19, 95, 95, 102, 97, 115, 116, 110, 98, 116, 95, 105, 110, 116, 95, 97,
        114, 114, 97, 121, 254, 255, 255, 241,
    ];
    assert!(from_bytes::<Value>(data).is_err());
}

#[test]
fn compound_using_nbt_token_key() {
    // Payload is a compound that has a key containing one of our NBT token
    // keys. We could in theory deserialize this, but instead we're going to
    // make the deserializer detect this and error.
    let data = &[
        10, 0, 0, 10, 0, 0, 10, 0, 0, 0, 10, 0, 0, 10, 0, 0, 10, 0, 0, 0, 10, 0, 0, 0, 0, 9, 0, 19,
        95, 95, 102, 97, 115, 116, 110, 98, 116, 95, 105, 110, 116, 95, 97, 114, 114, 97, 121, 2,
        0, 0, 0, 0, 9, 0, 10, 0, 110, 110, 98, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10,
        0, 0, 10, 0, 0, 10, 0, 0, 9, 0, 10, 0, 110,
    ];
    assert!(from_bytes::<Value>(data).is_err());
}
