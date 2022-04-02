use std::{collections::HashMap, iter::FromIterator};

use crate::{
    borrow, from_bytes,
    test::{resources::CHUNK_RAW_WITH_ENTITIES, Single, Wrap},
    to_bytes, ByteArray, IntArray, LongArray, Tag, Value,
};
use serde::Serialize;
use serde_bytes::Bytes;

use super::builder::Builder;

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
    let expected1 = Builder::new()
        .start_compound("")
        .int("a", 123)
        .int("b", 234)
        .end_compound()
        .build();
    let expected2 = Builder::new()
        .start_compound("")
        .int("b", 234)
        .int("a", 123)
        .end_compound()
        .build();
    assert!(expected1 == to_bytes(&v).unwrap() || expected2 == to_bytes(&v).unwrap());
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
fn value_hashmap() {
    // let v = Value::Unit;
    let v = Value::Compound(HashMap::from_iter([
        ("a".to_string(), Value::Int(123)),
        ("b".to_string(), Value::Byte(123)),
    ]));

    let expected1 = Builder::new()
        .start_compound("")
        .byte("b", 123)
        .int("a", 123)
        .end_compound()
        .build();
    let expected2 = Builder::new()
        .start_compound("")
        .int("a", 123)
        .byte("b", 123)
        .end_compound()
        .build();

    // hashmap order not predictable.
    assert!(expected1 == to_bytes(&v).unwrap() || expected2 == to_bytes(&v).unwrap());
}

#[test]
fn nbt_long_array_in_list() {
    let v = Single {
        val: [LongArray::new(vec![1, 2, 3]), LongArray::new(vec![4, 5, 6])],
    };

    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::LongArray, 2)
        .int_payload(3)
        .long_array_payload(&[1, 2, 3])
        .int_payload(3)
        .long_array_payload(&[4, 5, 6])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn borrowed_nbt_byte_array_in_list() {
    let v = Single {
        val: [
            borrow::ByteArray::new(&[1, 2, 3]),
            borrow::ByteArray::new(&[4, 5, 6]),
        ],
    };

    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::ByteArray, 2)
        .int_payload(3)
        .byte_array_payload(&[1, 2, 3])
        .int_payload(3)
        .byte_array_payload(&[4, 5, 6])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn borrowed_nbt_int_array_in_list() {
    let v = Single {
        val: [
            borrow::IntArray::new(&[1, 2, 3]),
            borrow::IntArray::new(&[4, 5, 6]),
        ],
    };

    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::IntArray, 2)
        .int_payload(3)
        .int_array_payload(&[1, 2, 3])
        .int_payload(3)
        .int_array_payload(&[4, 5, 6])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn borrowed_nbt_long_array_in_list() {
    let v = Single {
        val: [
            borrow::LongArray::new(&[1, 2, 3]),
            borrow::LongArray::new(&[4, 5, 6]),
        ],
    };

    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::LongArray, 2)
        .int_payload(3)
        .long_array_payload(&[1, 2, 3])
        .int_payload(3)
        .long_array_payload(&[4, 5, 6])
        .end_compound()
        .build();
    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn unit_enum() {
    #[derive(Serialize)]
    #[allow(unused)]
    enum Letter {
        A,
        B,
        C,
    }
    let v = Single { val: Letter::A };
    let expected = Builder::new()
        .start_compound("")
        .string("val", "A")
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn tuple_struct() {
    #[derive(Serialize)]
    struct Rgb(u8, u8, u8);
    let v = Single { val: Rgb(1, 2, 3) };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn enum_tuple_variant() {
    #[derive(Serialize)]
    enum Colour {
        Rgb(u8, u8, u8),
    }
    let v = Single {
        val: Colour::Rgb(1, 2, 3),
    };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn empty_list() {
    // Empty list ends up as a list of End tags, since there's no way to tell
    // what the type of the elements are, since serializing an element doesn't
    // happen for an empty list.

    let v = Single::<Vec<i32>> { val: vec![] };
    let actual = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::End, 0)
        .end_compound()
        .build();

    assert_eq!(expected, actual);
}

#[test]
#[ignore = "doesn't work yet"]
fn must_have_root() {
    // TODO: also do for other tag types.
    assert!(matches!(to_bytes(&123), Err(_)));
}

#[test]
fn list_of_empty_lists() {
    // Similar to the PostProcessing part of a chunk (at least in older
    // versions)
    let v = Single::<[[u32; 0]; 2]> { val: [[], []] };

    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::List, 2)
        .start_anon_list(Tag::End, 0)
        .start_anon_list(Tag::End, 0)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn round_trip() {
    // Deserialize, serialize, and deserialize again. The two Values should be
    // the same.
    let chunk: Value = from_bytes(CHUNK_RAW_WITH_ENTITIES).unwrap();
    let bytes = to_bytes(&chunk).unwrap();
    let roundtrip_chunk: Value = from_bytes(&bytes).unwrap();
    assert_eq!(roundtrip_chunk, chunk);
}

#[test]
fn serialize_bytes() {
    // Even serde_bytes gets serialized to just a list of bytes. Only the
    // dedicated fastnbt array types get serialized to NBT arrays.
    let v = Single {
        val: serde_bytes::Bytes::new(&[1, 2, 3]),
    };
    let expected = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn cesu_bytes() {
    // This unicode character is an example character that is different when
    // encoded with cesu8 and utf8.
    let symbol = "\u{10401}";
    let modified_unicode_str = cesu8::to_java_cesu8(symbol);

    #[derive(Serialize)]
    struct V {
        #[serde(rename = "\u{10401}")]
        val: String,
    }

    let v = V {
        val: symbol.to_string(),
    };

    let expected = Builder::new()
        .start_compound("")
        .tag(Tag::String)
        .name(symbol)
        .raw_len(modified_unicode_str.len())
        .raw_bytes(&modified_unicode_str)
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&v).unwrap());
}

#[test]
fn bytes_as_fields() {
    let mut map = HashMap::new();
    map.insert(Bytes::new(b"hello"), "world");

    let expected = Builder::new()
        .start_compound("")
        .string("hello", "world")
        .end_compound()
        .build();

    assert_eq!(expected, to_bytes(&map).unwrap());
}

#[test]
fn basic_newtype_variant_enum() {
    #[derive(Serialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum Letter {
        _A(u32),
        B(String),
    }

    #[derive(Serialize, Debug)]
    struct V {
        letter: Letter,
    }

    let v = V {
        letter: Letter::B("abc".to_owned()),
    };

    let expected = Builder::new()
        .start_compound("")
        .string("letter", "abc") // should deserialize as B?
        .end_compound()
        .build();

    let actual = to_bytes(&v).unwrap();

    assert_eq!(actual, expected);
}

// TODO: serialize_newtype_variant but for NOT NBT arrays
// TODO: deep nesting (doubts about how I'm managing state). Somewhat tackling
// this with fuzzing + arbitrary.
