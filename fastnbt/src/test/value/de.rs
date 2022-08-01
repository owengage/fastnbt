use std::borrow::Cow;

use serde::Deserialize;

use crate::{value::from_value, ByteArray, IntArray, LongArray};

#[test]
fn simple_types() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct V<'a> {
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
        str: Cow<'a, str>,
        string: String,
    }

    let val: V = from_value(&nbt!({
        "bool": true,
        "i8": i8::MAX,
        "i16": i16::MAX,
        "i32": i32::MAX,
        "i64": i64::MAX,
        "u8": u8::MAX,
        "u16": u16::MAX,
        "u32": u32::MAX,
        "u64": u64::MAX,
        "f32": f32::MAX,
        "f64": f64::MAX,
        "char": 'n',
        "str": "str",
        "string": "string",
    }))
    .unwrap();

    let expected = V {
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
        str: "str".into(),
        string: "string".to_string(),
    };

    assert_eq!(expected, val);
    // A borrowed string cannot be deserialized from Value
    assert!(matches!(val.str, Cow::Owned(_)));
}

#[test]
fn int_array_types() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct V {
        i128: i128,
        u128: u128,
        bytes: ByteArray,
        ints: IntArray,
        longs: LongArray,
    }

    let val = from_value(&nbt!({
        "i128": i128::MAX,
        "u128": u128::MAX,
        "bytes": [B; 1, 2, 3, 4, 5],
        "ints": [I; 1, 2, 3, 4, 5],
        "longs": [L; 1, 2, 3, 4, 5],
    }))
    .unwrap();

    let expected = V {
        i128: i128::MAX,
        u128: u128::MAX,
        bytes: ByteArray::new(vec![1, 2, 3, 4, 5]),
        ints: IntArray::new(vec![1, 2, 3, 4, 5]),
        longs: LongArray::new(vec![1, 2, 3, 4, 5]),
    };

    assert_eq!(expected, val);
}

#[test]
fn nested() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct V {
        list: Vec<i16>,
        nested: Inner,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct Inner {
        key: u8,
    }

    let val = from_value(&nbt!({
        "list": [1_i16, 2_i16],
        "nested": {
            "key": 42_u8,
        },
    }))
    .unwrap();

    let expected = V {
        list: vec![1, 2],
        nested: Inner { key: 42 },
    };

    assert_eq!(expected, val);
}

#[test]
fn no_root_compound() {
    assert_eq!(Ok(128_u8), from_value(&nbt!(128_u8)));
    assert_eq!(Ok('a'), from_value(&nbt!('a')));
    assert_eq!(Ok("string".to_string()), from_value(&nbt!("string")));
    assert_eq!(Ok(u128::MAX), from_value(&nbt!(u128::MAX)));
    assert_eq!(
        Ok(ByteArray::new(vec![1, 2, 3])),
        from_value(&nbt!([B; 1, 2, 3]))
    );
    assert_eq!(
        Ok(IntArray::new(vec![1, 2, 3])),
        from_value(&nbt!([I; 1, 2, 3]))
    );
    assert_eq!(
        Ok(LongArray::new(vec![1, 2, 3])),
        from_value(&nbt!([L; 1, 2, 3]))
    );
    assert_eq!(Ok(vec![1, 2, 3, 4]), from_value(&nbt!([1, 2, 3, 4])));
}
