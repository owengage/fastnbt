use std::{collections::HashMap, iter::FromIterator};

use crate::{ser::to_bytes, ByteArray, IntArray, LongArray, Tag};
use serde::{Deserialize, Serialize};

use super::builder::Builder;

#[derive(Serialize)]
struct Single<T: Serialize> {
    val: T,
}

#[derive(Serialize)]
struct Wrap<T: Serialize>(T);

#[test]
fn simple_byte() {
    let v = Single { val: 123u8 };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .byte("val", 123)
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}

#[test]
fn simple_numbers() {
    #[derive(Serialize)]
    struct V {
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
    }

    let v = V {
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
    };

    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .byte("i8", i8::MAX)
        .short("i16", i16::MAX)
        .int("i32", i32::MAX)
        .long("i64", i64::MAX)
        .byte("u8", u8::MAX as i8)
        .short("u16", u16::MAX as i16)
        .int("u32", u32::MAX as i32)
        .long("u64", u64::MAX as i64)
        .float("f32", f32::MAX)
        .double("f64", f64::MAX)
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}

#[test]
fn simple_string() {
    let v = Single {
        val: "hello".to_owned(),
    };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .string("val", "hello")
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}

#[test]
fn nested() {
    let v = Single {
        val: Single { val: 123 },
    };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .start_compound("val")
        .int("val", 123)
        .end_compound()
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}

#[test]
fn list_of_int() {
    let v = Single { val: [1i32, 2, 3] };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Int, 3)
        .int_payload(1)
        .int_payload(2)
        .int_payload(3)
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}

#[test]
fn list_of_short() {
    let v = Single { val: [1i16, 2, 3] };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Short, 3)
        .short_payload(1)
        .short_payload(2)
        .short_payload(3)
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}

#[test]
fn list_of_compounds() {
    #[derive(Serialize)]
    struct V {
        list: [Single<i32>; 3],
    }
    let v = V {
        list: [Single { val: 1 }, Single { val: 2 }, Single { val: 3 }],
    };
    let expected = Builder::new()
        .start_compound("")
        .start_list("list", Tag::Compound, 3)
        .int("val", 1)
        .end_compound()
        .int("val", 2)
        .end_compound()
        .int("val", 3)
        .end_compound()
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn list_of_list() {
    #[derive(Serialize)]
    struct V {
        list: [[Single<i32>; 1]; 3],
    }
    let v = V {
        list: [
            [Single { val: 1 }],
            [Single { val: 2 }],
            [Single { val: 3 }],
        ],
    };

    let expected = Builder::new()
        .start_compound("")
        .start_list("list", Tag::List, 3)
        // First list
        .start_anon_list(Tag::Compound, 1)
        .start_anon_compound()
        .int("val", 1)
        .end_anon_compound()
        // second
        .start_anon_list(Tag::Compound, 1)
        .start_anon_compound()
        .int("val", 2)
        .end_anon_compound()
        // third
        .start_anon_list(Tag::Compound, 1)
        .start_anon_compound()
        .int("val", 3)
        .end_anon_compound()
        // end outer compound
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn multiple_fields() {
    #[derive(Serialize)]
    struct V {
        a: u8,
        b: u16,
    }
    let v = V { a: 123, b: 1024 };
    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("")
        .tag(Tag::Byte)
        .name("a")
        .byte_payload(123)
        .tag(Tag::Short)
        .name("b")
        .short_payload(1024)
        .tag(Tag::End)
        .build();

    let expected = to_bytes(&v).unwrap();

    assert_eq!(expected, payload);
}

#[test]
fn serialize_str() {
    let v = Single { val: "hello" };
    let expected = Builder::new()
        .start_compound("")
        .string("val", "hello")
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn hashmap() {
    let v = HashMap::<_, _>::from_iter([("a", 123), ("b", 234)]);
    let expected = Builder::new()
        .start_compound("")
        .int("a", 123)
        .int("b", 234)
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn vec() {
    let v = Single {
        val: vec![1, 2, 3, 4, 5],
    };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Int, 5)
        .int_payload(1)
        .int_payload(2)
        .int_payload(3)
        .int_payload(4)
        .int_payload(5)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn vec_in_vec() {
    let v = Single {
        val: vec![vec![Single { val: 123 }]],
    };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::List, 1)
        .start_anon_list(Tag::Compound, 1)
        .int("val", 123)
        .end_anon_compound()
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn optional() {
    #[derive(Serialize)]
    struct V<'a> {
        opt1: Option<&'a str>,
        opt2: Option<&'a str>,
    }

    let v = V {
        opt1: Some("hello"),
        opt2: None,
    };

    let expected = Builder::new()
        .start_compound("")
        .string("opt1", "hello")
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());

    let v = Single { val: [v] };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Compound, 1)
        .start_anon_compound()
        .string("opt1", "hello")
        .end_anon_compound()
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn newtype_struct() {
    let v = Single { val: Wrap(123) };
    let expected = Builder::new()
        .start_compound("")
        .int("val", 123)
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn newtype_struct_list() {
    let v = Single {
        val: [Wrap(1), Wrap(2)],
    };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Int, 2)
        .int_payload(1)
        .int_payload(2)
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn nbt_byte_array() {
    let v = Single {
        val: ByteArray::new(vec![1, 2, 3]),
    };

    let expected = Builder::new()
        .start_compound("")
        .byte_array("val", &[1, 2, 3])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn nbt_int_array() {
    let v = Single {
        val: IntArray::new(vec![1, 2, 3]),
    };

    let expected = Builder::new()
        .start_compound("")
        .int_array("val", &[1, 2, 3])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn nbt_long_array() {
    let v = Single {
        val: LongArray::new(vec![1, 2, 3]),
    };

    let expected = Builder::new()
        .start_compound("")
        .long_array("val", &[1, 2, 3])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn hashmap_in_untagged_enum_causing_recurse_limit() {
    #[derive(Serialize)]
    #[serde(untagged)]
    pub enum Value {
        Compound(HashMap<String, Value>),
    }
    let v = Value::Compound(HashMap::new());

    let expected = Builder::new()
        .start_compound("")
        .byte("val", 123)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

// TODO: Test values without a root compound fail serialization.
// TODO: Borrowed arrays
// TODO: Arrays within lists
