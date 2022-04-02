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
