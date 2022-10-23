use fastnbt::{nbt, Value};

use crate::from_str;

fn assert_value(expected: &Value, snbt: &str) {
    let v: Value = from_str(snbt).unwrap();

    assert_eq!(*expected, v);
}

fn assert_all_values(expected: &Value, cases: &[&str]) {
    for case in cases {
        assert_value(expected, case)
    }
}

#[test]
fn simple_byte() {
    assert_all_values(
        &nbt!({
            "b": 123u8
        }),
        &[
            r#"{"b":123B}"#,
            r#"{ "b"  :  123B   }"#,
            r#"{'b': 123B}"#,
            r#"{'b': 123b}"#,
            r#"{b: 123b}"#,
            r#"  {
                'b': 123B
            }  "#,
        ],
    )
}

#[test]
fn mismatched_quote() {
    assert!(from_str::<Value>(r#"{"b':123B}"#).is_err());
    assert!(from_str::<Value>(r#"{'b":123B}"#).is_err());
    assert!(from_str::<Value>(r#"{'b:123B}"#).is_err());
}

#[test]
fn multiple_fields() {
    assert_all_values(
        &nbt!({
            "a": 123u8,
            "b": 0u8,
        }),
        &[
            r#"{"b":0b,"a":123B}"#,
            r#"{
                'a': 123b, 
                "b": 0b
            }"#,
        ],
    )
}

// TODO: Escapes of quote characters
