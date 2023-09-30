use std::collections::HashMap;

use serde::Serialize;

use crate::{to_value, ByteArray, IntArray, LongArray, Value};

#[test]
fn simple_types() {
    #[derive(Serialize)]
    struct V {
        bool: bool,
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
        f32: f32,
        f64: f64,
        char: char,
        str: &'static str,
        string: String,
    }

    let v = V {
        bool: true,
        i8: i8::MAX,
        i16: i16::MAX,
        i32: i32::MAX,
        i64: i64::MAX,
        u8: u8::MAX,
        u16: u16::MAX,
        u32: u32::MAX,
        u64: u64::MAX,
        f32: f32::MAX,
        f64: f64::MAX,
        char: 'n',
        str: "value",
        string: "value".to_string(),
    };

    let val = to_value(v).unwrap();
    // Note: we cannot use the nbt! macro here as that uses the `to_value` function
    let expected = Value::Compound(HashMap::from([
        ("bool".to_string(), Value::Byte(1)),
        ("i8".to_string(), Value::Byte(i8::MAX)),
        ("i16".to_string(), Value::Short(i16::MAX)),
        ("i32".to_string(), Value::Int(i32::MAX)),
        ("i64".to_string(), Value::Long(i64::MAX)),
        ("u8".to_string(), Value::Byte(u8::MAX as i8)),
        ("u16".to_string(), Value::Short(u16::MAX as i16)),
        ("u32".to_string(), Value::Int(u32::MAX as i32)),
        ("u64".to_string(), Value::Long(u64::MAX as i64)),
        ("f32".to_string(), Value::Float(f32::MAX)),
        ("f64".to_string(), Value::Double(f64::MAX)),
        ("char".to_string(), Value::Int('n' as i32)),
        ("str".to_string(), Value::String("value".to_string())),
        ("string".to_string(), Value::String("value".to_string())),
    ]));

    assert_eq!(expected, val);
}

#[test]
fn int_array_types() {
    #[derive(Serialize)]
    struct V {
        i128: i128,
        u128: u128,
        bytes: ByteArray,
        ints: IntArray,
        longs: LongArray,
    }

    let v = V {
        i128: i128::MAX,
        u128: u128::MAX,
        bytes: ByteArray::new(vec![1, 2, 3, 4, 5]),
        ints: IntArray::new(vec![1, 2, 3, 4, 5]),
        longs: LongArray::new(vec![1, 2, 3, 4, 5]),
    };

    let val = to_value(v).unwrap();
    let expected = Value::Compound(HashMap::from([
        (
            "i128".to_string(),
            // Only left most bit is 0
            Value::IntArray(IntArray::new(vec![
                i32::MAX,
                u32::MAX as i32,
                u32::MAX as i32,
                u32::MAX as i32,
            ])),
        ),
        (
            "u128".to_string(),
            // All bits are 1
            Value::IntArray(IntArray::new(vec![u32::MAX as i32; 4])),
        ),
        (
            "bytes".to_string(),
            Value::ByteArray(ByteArray::new(vec![1, 2, 3, 4, 5])),
        ),
        (
            "ints".to_string(),
            Value::IntArray(IntArray::new(vec![1, 2, 3, 4, 5])),
        ),
        (
            "longs".to_string(),
            Value::LongArray(LongArray::new(vec![1, 2, 3, 4, 5])),
        ),
    ]));

    assert_eq!(expected, val);
}

#[test]
fn nested() {
    #[derive(Serialize)]
    struct V {
        list: Vec<i16>,
        nested: Inner,
    }

    #[derive(Serialize)]
    struct Inner {
        key: u8,
    }

    let v = V {
        list: vec![1, 2],
        nested: Inner { key: 42 },
    };

    let val = to_value(v).unwrap();
    let expected = Value::Compound(HashMap::from([
        (
            "list".to_string(),
            Value::List(vec![Value::Short(1), Value::Short(2)]),
        ),
        (
            "nested".to_string(),
            Value::Compound(HashMap::from([("key".to_string(), Value::Byte(42))])),
        ),
    ]));

    assert_eq!(expected, val);
}

#[test]
fn no_root_compound() {
    assert_eq!(Ok(Value::Byte(-128)), to_value(-128_i8));
    assert_eq!(Ok(Value::Int(97)), to_value('a'));
    assert_eq!(Ok(Value::String("string".to_string())), to_value("string"));
    assert_eq!(
        Ok(Value::IntArray(IntArray::new(vec![u32::MAX as i32; 4]))),
        to_value(u128::MAX)
    );
    assert_eq!(
        Ok(Value::ByteArray(ByteArray::new(vec![1, 2, 3]))),
        to_value(ByteArray::new(vec![1, 2, 3]))
    );
    assert_eq!(
        Ok(Value::IntArray(IntArray::new(vec![1, 2, 3]))),
        to_value(IntArray::new(vec![1, 2, 3]))
    );
    assert_eq!(
        Ok(Value::LongArray(LongArray::new(vec![1, 2, 3]))),
        to_value(LongArray::new(vec![1, 2, 3]))
    );
    assert_eq!(
        Ok(Value::List(vec![
            Value::Byte(1),
            Value::Byte(2),
            Value::Byte(3),
            Value::Byte(4)
        ])),
        to_value(vec![
            Value::Byte(1),
            Value::Byte(2),
            Value::Byte(3),
            Value::Byte(4)
        ])
    );
}
