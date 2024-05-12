use std::{collections::HashMap, io::Cursor, iter::FromIterator};

use crate::{
    borrow, from_bytes, from_bytes_with_opts,
    test::{resources::CHUNK_RAW_WITH_ENTITIES, Single, Wrap},
    to_bytes, to_bytes_with_opts, to_writer_with_opts, ByteArray, DeOpts, IntArray, LongArray,
    SerOpts, Tag, Value,
};
use serde::{ser::SerializeMap, Deserialize, Serialize};
use serde_bytes::{ByteBuf, Bytes};
use serde_json::to_writer;

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
fn serialize_i128() {
    #[derive(Serialize)]
    struct V {
        max: u128,
        min: i128,
        zero: i128,
        counting: u128,
    }
    let v = V {
        max: u128::MAX,
        min: i128::MIN,
        zero: 0,
        counting: 1 << 96 | 2 << 64 | 3 << 32 | 4,
    };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        // All bits are 1
        .int_array("max", &[u32::MAX as i32; 4])
        // Only first bit is 1
        .int_array("min", &[1 << 31, 0, 0, 0])
        .int_array("zero", &[0; 4])
        .int_array("counting", &[1, 2, 3, 4])
        .tag(Tag::End)
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
fn list_of_list_of_bytes() {
    #[derive(Serialize)]
    struct V {
        list: [[u8; 1]; 3],
    }
    let v = V {
        list: [[1], [2], [3]],
    };

    let expected = Builder::new()
        .start_compound("")
        .start_list("list", Tag::List, 3) // we're missing the element tag and size here.
        // First list
        .start_anon_list(Tag::Byte, 1)
        .byte_payload(1)
        // second
        .start_anon_list(Tag::Byte, 1)
        .byte_payload(2)
        // third
        .start_anon_list(Tag::Byte, 1)
        .byte_payload(3)
        // end outer compound
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
    assert!(to_bytes(&123).is_err());
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
        .raw_str_len(modified_unicode_str.len())
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

#[test]
fn reduced_roundtrip_bug() {
    // When you have an empty compound, MapSerialize doesn't call
    // serialize_entry at all, which makes sense. This would be fine except we
    // delay writing the tag and name of the compound until the first key-value
    // entry is being written. This delay is because NBT arrays are modelled in
    // serde as maps with a special key-value entry, and we don't want to
    // serialize the compound tag if it turns out to be a NBT array instead.
    //
    // This however means that if an empty compound is serialized, that
    // serialize_entry doesn't get called and we end up not writing the compound
    // tag or name as a result. The fix is to detect in the end() call that we
    // haven't written the header, and write it.

    let payload = Builder::new()
        .start_compound("")
        .start_list("aaa", Tag::Compound, 1)
        .start_anon_compound()
        .end_anon_compound()
        .end_compound()
        .build();

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Empty {}

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct V {
        aaa: Vec<Empty>,
    }

    // Chunk looks fine as judged by println
    let v: V = from_bytes(&payload).unwrap();

    // This potentially broken?
    let bs = to_bytes(&v).unwrap();

    let roundtrip: V = from_bytes(&bs).unwrap();
    assert_eq!(roundtrip, v);
}

#[test]
fn empty_compound_root() {
    #[derive(Serialize)]
    struct Empty {}

    #[derive(Serialize)]
    struct Wrapper(Empty);

    let payload = Builder::new().start_compound("").end_compound().build();
    let bs1 = to_bytes(&Empty {}).unwrap();
    let bs2 = to_bytes(&Wrapper(Empty {})).unwrap();
    assert_eq!(payload, bs1);
    assert_eq!(payload, bs2);
}

#[test]
fn no_root() {
    assert!(to_bytes(&123_u8).is_err());
    assert!(to_bytes(&123_u16).is_err());
    assert!(to_bytes(&123_u32).is_err());
    assert!(to_bytes(&123_u64).is_err());
    assert!(to_bytes(&123_u128).is_err());
    assert!(to_bytes(&123_i8).is_err());
    assert!(to_bytes(&123_i16).is_err());
    assert!(to_bytes(&123_i32).is_err());
    assert!(to_bytes(&123_i64).is_err());
    assert!(to_bytes(&123_i128).is_err());
    assert!(to_bytes(&true).is_err());
    assert!(to_bytes(&"hello").is_err());
    assert!(to_bytes(&vec![1]).is_err());

    assert!(to_bytes(&HashMap::<&str, ()>::new()).is_ok());
}

#[test]
fn list_of_bytebuf() {
    #[derive(Serialize)]
    struct V {
        list: [ByteBuf; 1],
    }

    let v = V {
        list: [ByteBuf::from([1, 2, 3])],
    };
    let actual = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .start_list("list", Tag::List, 1)
        .start_anon_list(Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    assert_eq!(expected, actual);
}

#[test]
fn list_with_none_errors() {
    #[derive(Serialize)]
    struct V {
        list: [Option<u8>; 1],
    }
    let v = V { list: [None] };
    assert!(to_bytes(&v).is_err());
}

#[test]
fn unit_errors() {
    #[derive(Serialize)]
    struct V {
        unit: (),
    }
    #[derive(Serialize)]
    struct VS {
        unit: [(); 1],
    }
    let v = V { unit: () };
    let vs = VS { unit: [()] };
    assert!(to_bytes(&v).is_err());
    assert!(to_bytes(&vs).is_err());
}

#[test]
fn unit_struct_errors() {
    #[derive(Serialize)]
    struct Unit;

    #[derive(Serialize)]
    struct V {
        unit: Unit,
    }
    #[derive(Serialize)]
    struct VS {
        unit: [Unit; 1],
    }
    let v = V { unit: Unit };
    let vs = VS { unit: [Unit] };
    assert!(to_bytes(&v).is_err());
    assert!(to_bytes(&vs).is_err());
}

#[test]
fn serialize_key_and_value() {
    struct Dummy;
    impl Serialize for Dummy {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(None)?;
            map.serialize_key("test")?;
            map.serialize_value("value")?;
            map.end()
        }
    }

    let bs = to_bytes(&Dummy).unwrap();
    let actual = to_bytes(&nbt!({"test":"value"})).unwrap();
    assert_eq!(actual, bs);
}

#[test]
fn serialize_root_with_name() {
    #[derive(Serialize)]
    struct Empty {}
    let expected = Builder::new().start_compound("test").end_compound().build();
    let opts = SerOpts::new().root_name("test");
    let mut actual_via_writer = Cursor::new(Vec::new());
    to_writer_with_opts(&mut actual_via_writer, &Empty {}, opts.clone()).unwrap();

    let actual_via_bytes = to_bytes_with_opts(&Empty {}, opts.clone()).unwrap();
    let actual_value = to_bytes_with_opts(&Value::Compound(HashMap::new()), opts.clone()).unwrap();

    assert_eq!(actual_via_bytes, expected);
    assert_eq!(actual_via_writer.into_inner(), expected);
    assert_eq!(actual_value, expected);
}

#[test]
fn serialize_networked_compound() {
    #[derive(Serialize)]
    struct Example {
        networked: String,
    }

    let data = Example {
        networked: "compound".to_string(),
    };
    let bytes: Vec<u8> = to_bytes_with_opts(&data, SerOpts::network_nbt()).unwrap();

    assert_eq!(
        bytes,
        b"\
        \x0a\
            \x08\
                \x00\x09networked\
                \x00\x08compound\
        \x00"
    );
}
