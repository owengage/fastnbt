use crate::{de::from_bytes, Tag, Value};

use super::builder::Builder;

// Given a v: Value, a key: str, and a pattern, check the value is a compound
// withat key and it's value matches the pattern. Optionally add a condition for the
// matched value
macro_rules! assert_contains {
    ($v:ident, $key:expr, $p:pat) => {
        if let Value::Compound(v) = &$v {
            match v[$key] {
                $p => {}
                _ => panic!("expected Some({}), got {:?}", stringify!($p), v.get($key)),
            }
        } else {
            panic!("expected compound");
        }
    };
    ($v:ident, $key:expr, $p:pat, $check:expr) => {
        if let Value::Compound(v) = &$v {
            match v[$key] {
                $p => assert!($check),
                _ => panic!("expected Some({}), got {:?}", stringify!($p), v.get($key)),
            }
        } else {
            panic!("expected compound");
        }
    };
}

#[test]
fn distinguish_byte() {
    let input = Builder::new()
        .start_compound("")
        .byte("a", 123)
        .byte("b", -123)
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::Byte(123));
    assert_contains!(v, "b", Value::Byte(-123));
}

#[test]
fn distinguish_short() {
    let input = Builder::new()
        .start_compound("")
        .short("a", 1)
        .short("b", 1000)
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::Short(1));
    assert_contains!(v, "b", Value::Short(1000));
}

#[test]
fn distinguish_int() {
    let input = Builder::new()
        .start_compound("")
        .int("a", 1)
        .int("b", 1000)
        .int("c", 1_000_000)
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::Int(1));
    assert_contains!(v, "b", Value::Int(1000));
    assert_contains!(v, "c", Value::Int(1_000_000));
}

#[test]
fn distinguish_long() {
    let input = Builder::new()
        .start_compound("")
        .long("a", 1)
        .long("b", 1000)
        .long("c", 1_000_000)
        .long("d", 10_000_000_000)
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::Long(1));
    assert_contains!(v, "b", Value::Long(1000));
    assert_contains!(v, "c", Value::Long(1_000_000));
    assert_contains!(v, "d", Value::Long(10_000_000_000));
}

#[test]
fn distinguish_floats() {
    let input = Builder::new()
        .start_compound("")
        .float("a", 1.23)
        .double("b", 3.21)
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::Float(f), f == 1.23);
    assert_contains!(v, "b", Value::Double(f), f == 3.21);
}

#[test]
fn distinguish_string() {
    let input = Builder::new()
        .start_compound("")
        .string("a", "hello")
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::String(ref s), s == "hello");
}

#[test]
fn distinguish_arrays() {
    let input = Builder::new()
        .start_compound("")
        .byte_array("a", &[1, 2, 3])
        .int_array("b", &[4, 5, 6])
        .long_array("c", &[7, 8, 9])
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(
        v,
        "a",
        Value::ByteArray(ref data),
        data.iter().eq(&[1, 2, 3])
    );
    assert_contains!(
        v,
        "b",
        Value::IntArray(ref data),
        data.iter().eq(&[4, 5, 6])
    );
    assert_contains!(
        v,
        "c",
        Value::LongArray(ref data),
        data.iter().eq(&[7, 8, 9])
    );
}

#[test]
fn distinguish_lists() {
    let input = Builder::new()
        .start_compound("")
        .start_list("a", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .start_list("b", Tag::Int, 3)
        .int_payload(1)
        .int_payload(2)
        .int_payload(3)
        .start_list("c", Tag::Long, 3)
        .long_payload(1)
        .long_payload(2)
        .long_payload(3)
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(
        v,
        "a",
        Value::List(ref data),
        data.iter()
            .eq(&[Value::Byte(1), Value::Byte(2), Value::Byte(3)])
    );
    assert_contains!(
        v,
        "b",
        Value::List(ref data),
        data.iter()
            .eq(&[Value::Int(1), Value::Int(2), Value::Int(3)])
    );
    assert_contains!(
        v,
        "c",
        Value::List(ref data),
        data.iter()
            .eq(&[Value::Long(1), Value::Long(2), Value::Long(3)])
    );
}

#[test]
fn distinguish_compound() {
    let input = Builder::new()
        .start_compound("")
        .start_compound("a")
        .end_compound()
        .end_compound()
        .build();

    let v: Value = from_bytes(&input).unwrap();
    assert_contains!(v, "a", Value::Compound(_));
}
