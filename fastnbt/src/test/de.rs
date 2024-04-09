use std::{borrow::Cow, collections::HashMap, io::Cursor};

use serde::{Deserialize, Serialize};

use crate::{
    borrow,
    error::{Error, Result},
    from_bytes, from_bytes_with_opts, from_reader, nbt,
    test::builder::Builder,
    to_bytes, ByteArray, DeOpts, IntArray, LongArray, Tag, Value,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Single<T: Serialize> {
    val: T,
}

fn from_all<'de, T>(payload: &'de [u8]) -> T
where
    T: Deserialize<'de> + PartialEq + std::fmt::Debug,
{
    let v_bytes: T = from_bytes(payload).unwrap();
    let v_read: T = from_reader(payload).unwrap();
    assert_eq!(v_bytes, v_read);
    v_bytes
}

#[test]
fn error_impls_sync_send() {
    fn i<T: Clone + Send + Sync + std::error::Error>(_: T) {}
    i(Error::invalid_tag(1));
}

#[test]
fn descriptive_error_on_gzip_magic() {
    let r = from_bytes::<()>(&[0x1f, 0x8b]);
    assert!(matches!(r, Result::Err(_)));
    let e = r.unwrap_err();
    assert!(e.to_string().to_lowercase().contains("gzip"));
}

#[test]
fn simple_byte() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        abc: i8,
        def: i8,
    }

    let payload = Builder::new()
        .start_compound("")
        .byte("abc", 123)
        .byte("def", 111)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(v.abc, 123);
    assert_eq!(v.def, 111);
}

#[test]
fn simple_floats() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct V {
        f: f32,
        d: f64,
    }

    let payload = Builder::new()
        .start_compound("object")
        .float("f", 1.23)
        .double("d", 2.34)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(v.f, 1.23);
    assert_eq!(v.d, 2.34);
}

#[test]
fn simple_shorts() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        abc: i16,
        def: u16,
    }

    let payload = Builder::new()
        .start_compound("")
        .short("abc", 256)
        .short("def", 257)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(v.abc, 256);
    assert_eq!(v.def, 257);
}

#[test]
fn short_to_u16_out_of_range_errors() {
    #[derive(Deserialize)]
    struct V {
        _abc: u16,
    }

    let payload = Builder::new()
        .start_compound("")
        .short("_abc", -123)
        .end_compound()
        .build();

    let v: Result<V> = from_bytes(payload.as_slice());
    assert!(v.is_err());
}

#[test]
fn multiple_fields() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        a: u8,
        b: u16,
    }

    let payload = Builder::new()
        .start_compound("")
        .byte("a", 123)
        .short("b", 1024)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
    assert_eq!(v.b, 1024);
    Ok(())
}

#[test]
fn numbers_into_u32() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        a: u32,
        b: u32,
        c: u32,
    }

    let payload = Builder::new()
        .start_compound("")
        .byte("a", 123)
        .short("b", 2 << 8)
        .int("c", 2 << 24)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(v.a, 123);
    assert_eq!(v.b, 2 << 8);
    assert_eq!(v.c, 2 << 24);
}

#[test]
fn string_into_ref_str() {
    #[derive(Deserialize)]
    struct V<'a> {
        a: &'a str,
    }

    let payload = Builder::new()
        .start_compound("")
        .string("a", "hello")
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert_eq!("hello", v.a);

    // Cannot borrow if from a reader.
    assert!(from_reader::<_, V>(payload.as_slice()).is_err());
}

#[test]
fn string_into_string() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        a: String,
    }

    let payload = Builder::new()
        .start_compound("")
        .string("a", "hello")
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!("hello", v.a);
}

#[test]
fn nested_compound() {
    #[derive(Deserialize)]
    struct Nested {
        b: u32,
    }

    #[derive(Deserialize)]
    struct V {
        a: u32,
        nested: Nested,
    }

    let payload = Builder::new()
        .start_compound("")
        .byte("a", 123)
        .start_compound("nested")
        .byte("b", 1)
        .end_compound()
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
    assert_eq!(v.nested.b, 1);
}

#[test]
fn unwanted_primative_payloads() {
    #[derive(Deserialize)]
    struct V {
        a: u32,
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte("a", 123)
        .short("b", 1)
        .int("c", 2)
        .long("d", 3)
        .string("e", "test")
        .float("f", 1.23)
        .double("g", 2.34)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
}

#[test]
fn simple_hashmap() {
    let payload = Builder::new()
        .start_compound("object")
        .int("a", 1)
        .int("b", 2)
        .end_compound()
        .build();

    let v: HashMap<&str, i32> = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v["a"], 1);
    assert_eq!(v["b"], 2);
}

#[test]
fn simple_hashmap_with_untagged_enum() {
    let payload = Builder::new()
        .start_compound("object")
        .int("a", 1)
        .string("b", "2")
        .end_compound()
        .build();

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum E<'a> {
        Int(i32),
        String(&'a str),
    }

    let v: HashMap<&str, E> = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v["a"], E::Int(1));
    assert_eq!(v["b"], E::String("2"));
}

#[test]
fn nested_hashmaps_with_enums() {
    let payload = Builder::new()
        .start_compound("object")
        .int("a", 1)
        .start_compound("b")
        .int("inner", 2)
        .end_compound()
        .end_compound()
        .build();

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(untagged)]
    enum E<'a> {
        Int(i32),
        #[serde(borrow)]
        Map(HashMap<&'a str, i32>),
    }

    let v: HashMap<&str, E> = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v["a"], E::Int(1));
    match v["b"] {
        E::Map(ref map) => assert_eq!(map["inner"], 2),
        _ => panic!(),
    }
}

#[test]
fn simple_list() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        a: Vec<u32>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("a", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(v.a, [1, 2, 3]);
}

#[test]
fn optional() {
    #[derive(Deserialize)]
    struct V<'a> {
        opt1: Option<&'a str>,
        opt2: Option<&'a str>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .string("opt1", "hello")
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.opt1, Some("hello"));
    assert_eq!(v.opt2, None);
}

#[test]
fn list_of_compounds() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Inner {
        a: u32,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    struct V {
        inner: Option<Vec<Inner>>,
        after: i8,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("inner", Tag::Compound, 3)
        .byte("a", 1)
        .start_compound("ignored")
        .end_compound()
        .end_compound()
        .byte("a", 2)
        .end_compound()
        .byte("a", 3)
        .end_compound()
        .byte("after", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(
        v.inner,
        Some(vec![Inner { a: 1 }, Inner { a: 2 }, Inner { a: 3 }])
    );
    assert_eq!(v.after, 123);
}

#[test]
fn complex_nesting() {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Inner {
        a: u32,
        b: Option<Vec<i32>>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        inner: Vec<Inner>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("inner", Tag::Compound, 3)
        .byte("a", 1)
        .start_list("b", Tag::Int, 2)
        .int_payload(1)
        .int_payload(2)
        .start_compound("ignored")
        .end_compound()
        .end_compound()
        .byte("a", 2)
        .end_compound()
        .byte("a", 3)
        .end_compound()
        .byte("after", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(
        v.inner,
        vec![
            Inner {
                a: 1,
                b: Some(vec![1, 2])
            },
            Inner { a: 2, b: None },
            Inner { a: 3, b: None }
        ]
    );
}

#[test]
fn unit_just_requires_presense() {
    #[derive(Deserialize)]
    struct Foo;

    #[derive(Deserialize)]
    struct V {
        _unit: (),
        _unit_struct: Foo,
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte("_unit", 0)
        .byte("_unit_struct", 0)
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_ok());
}

#[test]
fn unit_not_present_errors() {
    #[derive(Deserialize)]
    struct V {
        _unit: (),
    }

    let payload = Builder::new()
        .start_compound("object")
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_err());
}

#[test]
fn ignore_compound() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        a: u8,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_compound("inner")
        .byte("ignored", 1)
        .end_compound()
        .start_compound("inner")
        .byte("ignored", 1)
        .end_compound()
        .byte("a", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());
    assert_eq!(v.a, 123);
}

#[test]
fn ignore_primitives_in_ignored_compound() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        a: u8,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_compound("ignoreall")
        .float("ignored", 1.23)
        .double("ignored", 1.234)
        .byte("ig", 1)
        .short("ig", 2)
        .int("ig", 3)
        .long("ig", 4)
        .string("ig", "hello")
        .end_compound()
        .byte("a", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());
    assert_eq!(v.a, 123);
}

#[test]
fn ignore_list() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct V {
        a: u8,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("ignored", Tag::Byte, 2)
        .byte_payload(1)
        .byte_payload(2)
        .byte("a", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());
    assert_eq!(v.a, 123);
}

#[test]
fn ignore_list_of_compound() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct V {
        a: u8,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("ignored", Tag::Compound, 2)
        .byte("a", 1) // ignored!
        .end_compound()
        .end_compound()
        .byte("a", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());
    assert_eq!(v.a, 123);
}

#[test]
fn byte_array_from_list_bytes() {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("arr", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v.arr, [1, 2, 3]);
}

#[test]
fn byte_array_from_nbt_short_list() -> Result<()> {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("arr", Tag::Short, 3)
        .short_payload(1)
        .short_payload(2)
        .short_payload(3)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [0, 1, 0, 2, 0, 3]);

    Ok(())
}

#[test]
fn byte_array_from_nbt_int_list() {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("arr", Tag::Int, 2)
        .int_payload(1)
        .int_payload(2)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v.arr, [0, 0, 0, 1, 0, 0, 0, 2]);
}

#[test]
fn byte_array_from_nbt_long_list() {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("arr", Tag::Long, 2)
        .long_payload(1)
        .long_payload(2)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v.arr, [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);
}

#[test]
fn newtype_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Inner(u8);

    #[derive(Deserialize, Debug, PartialEq)]
    struct V {
        a: Inner,
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte("a", 123)
        .end_compound()
        .build();

    let v: V = from_all(payload.as_slice());
    assert_eq!(v.a.0, 123);
}

#[test]
fn vec_from_nbt_byte_array() {
    #[derive(Deserialize)]
    struct V {
        a: ByteArray,
        b: IntArray,
        c: LongArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::ByteArray)
        .name("a")
        .int_payload(3)
        .byte_array_payload(&[1, 2, 3])
        .tag(Tag::IntArray)
        .name("b")
        .int_payload(3)
        .int_array_payload(&[4, 5, 6])
        .tag(Tag::LongArray)
        .name("c")
        .int_payload(3)
        .long_array_payload(&[7, 8, 9])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert!(v.a.iter().eq(&[1, 2, 3]));
    assert_eq!(*v.b, [4, 5, 6]);
    assert_eq!(*v.c, [7, 8, 9]);
}

#[derive(Deserialize)]
struct Blockstates<'a>(&'a [u8]);

#[test]
fn blockstates() {
    #[derive(Deserialize)]
    struct V<'a> {
        #[serde(borrow)]
        states: Blockstates<'a>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::LongArray)
        .name("states")
        .int_payload(3)
        .long_payload(1)
        .long_payload(2)
        .long_payload(3)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(
        [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3],
        v.states.0
    );
}

#[test]
fn ignore_integral_arrays() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct V {}

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::ByteArray)
        .name("a")
        .int_payload(3)
        .byte_array_payload(&[1, 2, 3])
        .tag(Tag::IntArray)
        .name("b")
        .int_payload(3)
        .int_array_payload(&[4, 5, 6])
        .tag(Tag::LongArray)
        .name("c")
        .int_payload(3)
        .long_array_payload(&[7, 8, 9])
        .end_compound()
        .build();

    from_all::<V>(payload.as_slice());
}

#[test]
fn fixed_array() {
    #[derive(Deserialize)]
    struct Inner<'a> {
        a: &'a [u8],
    }
    #[derive(Deserialize)]
    pub struct Level<'a> {
        #[serde(borrow)]
        inner: [Inner<'a>; 3],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("inner", Tag::Compound, 3)
        .byte_array("a", &[1, 2, 3])
        .end_compound()
        .byte_array("a", &[4, 5, 6])
        .end_compound()
        .byte_array("a", &[7, 8, 9])
        .end_compound() // end of list
        .end_compound() // end of outer compound
        .build();

    let v: Level = from_bytes(payload.as_slice()).unwrap();
    assert_eq!([1, 2, 3], v.inner[0].a);
    assert_eq!([4, 5, 6], v.inner[1].a);
    assert_eq!([7, 8, 9], v.inner[2].a);
}

#[test]
fn type_mismatch_string() -> Result<()> {
    #[derive(Deserialize, Debug)]
    pub struct V {
        _a: String,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int("_a", 123)
        .end_compound() // end of outer compound
        .build();

    let res = from_bytes::<V>(payload.as_slice());

    assert!(res.is_err());
    Ok(())
}

#[test]
fn basic_palette_item() {
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    pub struct PaletteItem {
        name: String,
        #[allow(dead_code)]
        properties: HashMap<String, String>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_compound("Properties")
        .string("lit", "false")
        .end_compound()
        .string("Name", "minecraft:redstone_ore")
        .end_compound()
        .build();

    let res: PaletteItem = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(res.name, "minecraft:redstone_ore");
}

#[test]
fn basic_unit_variant_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Letter {
        #[serde(rename = "a")]
        A,
        B,
        C,
    }

    #[derive(Deserialize, Debug)]
    struct V {
        letter: Letter,
    }
    let payload = Builder::new()
        .start_compound("")
        .string("letter", "a")
        .end_compound()
        .build();

    let res: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(res.letter, Letter::A);
}

#[test]
fn basic_newtype_variant_enum() {
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum Letter {
        A(u32),
        B(String),
    }

    #[derive(Deserialize, Debug)]
    struct V {
        letter: Letter,
    }
    let payload = Builder::new()
        .start_compound("")
        .string("letter", "abc") // should deserialize as B?
        .end_compound()
        .build();

    let res: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(res.letter, Letter::B("abc".to_owned()));
}

#[test]
fn unit_variant_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        A,
        B,
        C,
    }
    #[derive(Deserialize)]
    struct V {
        e1: E,
        e2: E,
        e3: E,
    }

    let payload = Builder::new()
        .start_compound("object")
        .string("e1", "A")
        .string("e2", "B")
        .string("e3", "C")
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert_eq!(v.e1, E::A);
    assert_eq!(v.e2, E::B);
    assert_eq!(v.e3, E::C);
}

#[test]
fn integrals_in_fullvalue() {
    let payload = Builder::new()
        .start_compound("object")
        .int("a", 1)
        .int("b", 2)
        .end_compound()
        .build();

    let v: Value = from_bytes(payload.as_slice()).unwrap();
    match v {
        Value::Compound(ref map) => {
            let a = &map["a"];
            match a {
                Value::Int(i) => assert_eq!(*i, 1),
                _ => panic!("{:?}", a),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn floating_in_fullvalue() {
    let payload = Builder::new()
        .start_compound("object")
        .float("a", 1.0)
        .double("b", 2.0)
        .float("c", 3.0)
        .end_compound()
        .build();

    let val: Value = from_bytes(payload.as_slice()).unwrap();
    match val {
        Value::Compound(ref map) => {
            let a = &map["a"];
            match a {
                Value::Float(f) => assert_eq!(*f, 1.0),
                _ => panic!("{:?}", a),
            }
            let b = &map["b"];
            match b {
                Value::Double(f) => assert_eq!(*f, 2.0),
                _ => panic!("{:?}", a),
            }
            let c = &map["c"];
            match c {
                Value::Float(f) => assert_eq!(*f, 3.0),
                _ => panic!("{:?}", a),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn byte_array_in_fullvalue() {
    let payload = Builder::new()
        .start_compound("object")
        .byte_array("a", &[1, 2, 3])
        .end_compound()
        .build();

    let v: Value = from_bytes(payload.as_slice()).unwrap();
    match v {
        Value::Compound(ref map) => {
            let a = &map["a"];
            match a {
                Value::ByteArray(arr) => assert!(arr.iter().eq(&[1, 2, 3])),
                _ => panic!("{:?}", a),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn int_array_in_fullvalue() {
    let payload = Builder::new()
        .start_compound("object")
        .int_array("a", &[1, 2, 3])
        .end_compound()
        .build();

    let v: Value = from_bytes(payload.as_slice()).unwrap();
    match v {
        Value::Compound(ref map) => {
            let a = &map["a"];
            match a {
                Value::IntArray(arr) => assert_eq!(&**arr, &[1, 2, 3]),
                _ => panic!("incorrect value: {:?}", a),
            }
        }
        _ => panic!(),
    }
}

#[test]
fn trailing_bytes() {
    // Can't really see a way to assert that there are no trailing bytes. We
    // don't return how far in to the input we got.
    let mut input = Builder::new().start_compound("").end_compound().build();
    input.push(1);
    let _v: Value = from_bytes(&input).unwrap();
}

#[test]
fn cesu8_string_in_nbt() {
    // In the builder we always convert to java cesu8 form for strings anyway,
    // but this test is more explicit and includes some unicode that actually
    // has a different representation in cesu8 and utf-8.
    let modified_unicode_str = cesu8::to_java_cesu8("😈");

    let input = Builder::new()
        .start_compound("")
        .tag(Tag::String)
        .name("hello")
        .raw_str_len(modified_unicode_str.len())
        .raw_bytes(&modified_unicode_str)
        .end_compound()
        .build();

    let _v: Value = from_bytes(&input).unwrap();
}

#[test]
fn cannot_borrow_cesu8_if_diff_repr() {
    #[derive(Deserialize, Debug)]
    pub struct V<'a> {
        _name: &'a str,
    }

    let modified_unicode_str = cesu8::to_java_cesu8("😈");

    let input = Builder::new()
        .start_compound("")
        .tag(Tag::String)
        .name("_name")
        .raw_str_len(modified_unicode_str.len())
        .raw_bytes(&modified_unicode_str)
        .end_compound()
        .build();

    let v: Result<V> = from_bytes(&input);
    assert!(v.is_err());
}

#[test]
fn can_borrow_cesu8_if_same_repr() {
    #[derive(Deserialize, Debug)]
    pub struct V<'a> {
        name: &'a str,
    }

    let modified_unicode_str = cesu8::to_java_cesu8("abc");

    let input = Builder::new()
        .start_compound("")
        .tag(Tag::String)
        .name("name")
        .raw_str_len(modified_unicode_str.len())
        .raw_bytes(&modified_unicode_str)
        .end_compound()
        .build();

    let v: Result<V> = from_bytes(&input);
    assert!(v.is_ok());
    assert_eq!("abc", v.unwrap().name);
}

#[test]
fn can_cow_cesu8() {
    #[derive(Deserialize, Debug)]
    pub struct V<'a> {
        owned: Cow<'a, str>,
        #[serde(borrow, deserialize_with = "crate::borrow::deserialize_cow_str")]
        borrowed: Cow<'a, str>,
    }

    let modified_unicode_str = cesu8::to_java_cesu8("😈");

    let input = Builder::new()
        .start_compound("")
        .tag(Tag::String)
        .name("owned")
        .raw_str_len(modified_unicode_str.len())
        .raw_bytes(&modified_unicode_str)
        .string("borrowed", "abc")
        .end_compound()
        .build();

    let v: V = from_bytes(&input).unwrap();
    assert!(matches!(v.owned, Cow::Owned(_)));
    assert_eq!("😈", v.owned);

    assert!(matches!(v.borrowed, Cow::Borrowed(_)));
    assert_eq!("abc", v.borrowed);
}

#[test]
fn large_list() {
    let input = [10, 0, 0, 9, 0, 0, 10, 4, 0, 5, 252];
    let _v: Result<Value> = from_bytes(&input);
}

#[test]
fn hashmap_with_bytes() {
    // Users should be able to decode strings as borrowed byte strings if they
    // really want to.
    let input = Builder::new()
        .start_compound("")
        .string("hello", "😈")
        .end_compound()
        .build();

    let v: HashMap<&[u8], &[u8]> = from_bytes(&input).unwrap();
    assert_eq!(
        cesu8::from_java_cesu8(v["hello".as_bytes()])
            .unwrap()
            .as_bytes(),
        "😈".as_bytes()
    );
}

#[test]
fn hashmap_with_byte_buf() {
    // Users should be able to decode strings as borrowed byte strings if they
    // really want to.
    let input = Builder::new()
        .start_compound("")
        .string("hello", "😈")
        .end_compound()
        .build();

    let _v: HashMap<&[u8], serde_bytes::ByteBuf> = from_bytes(&input).unwrap();
}

#[test]
fn chars() {
    let input = Builder::new()
        .start_compound("")
        .string("val", "a")
        .end_compound()
        .build();
    let v: Single<char> = from_bytes(&input).unwrap();
    assert_eq!('a', v.val);
}

#[test]
fn enum_variant_types() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Letter {
        NewType(u32),
        Tuple(u8, u8, u8),
        Struct { a: String },
    }

    let newtype_input = Builder::new()
        .start_compound("")
        .string("val", "NewType")
        .end_compound()
        .build();
    let v: Result<Single<Letter>> = from_bytes(&newtype_input);
    assert!(v.is_err());

    let tuple_input = Builder::new()
        .start_compound("")
        .string("val", "Tuple")
        .end_compound()
        .build();

    let v: Result<Single<Letter>> = from_bytes(&tuple_input);
    assert!(v.is_err());

    let struct_input = Builder::new()
        .start_compound("")
        .string("val", "Struct")
        .end_compound()
        .build();
    let v: Result<Single<Letter>> = from_bytes(&struct_input);
    assert!(v.is_err());
}

#[test]
fn tuple_struct() {
    #[derive(Deserialize, Serialize)]
    struct Rgb(u8, u8, u8);

    let input = Builder::new()
        .start_compound("")
        .start_list("val", Tag::Byte, 3)
        .byte_payload(1)
        .byte_payload(2)
        .byte_payload(3)
        .end_compound()
        .build();

    let v: Single<Rgb> = from_bytes(&input).unwrap();
    assert!(matches!(v.val, Rgb(1, 2, 3)));
}

#[test]
fn nested_tuple_list() {
    // https://github.com/owengage/fastnbt/issues/63

    // Reason for problem described here:
    // https://github.com/serde-rs/serde/issues/1557

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct IntTuple {
        value: (i32, i32, i32),
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct IntTupleList {
        list: Vec<IntTuple>,
    }

    let nbt_list = nbt!({
        "list": [{
            "value": [1, 2, 3],
        }],
    });

    let v: IntTupleList = from_bytes(&to_bytes(&nbt_list).unwrap()).unwrap();
    // let v: IntTupleList = from_bytes(&built).unwrap();
    assert_eq!(v.list[0].value, (1, 2, 3))
}

#[test]
fn pathological_tuples() {
    // Reason for problem described here:
    // https://github.com/serde-rs/serde/issues/1557

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct IntTuple {
        value: ((i32, i32, i32), (i32, i32)),
    }

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct IntTupleList {
        list: Vec<IntTuple>,
    }

    let nbt_list = nbt!({
        "list": [{
            "value": [[1, 2, 3],[4,5]]
        }],
    });

    let v: IntTupleList = from_bytes(&to_bytes(&nbt_list).unwrap()).unwrap();
    assert_eq!(v.list[0].value, ((1, 2, 3), (4, 5)))
}

#[test]
fn byte_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        bs: ByteArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte_array("bs", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert!(v.bs.iter().eq(&[1, 2, 3, 4, 5]));

    Ok(())
}

#[test]
fn int_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        is: IntArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int_array("is", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(&*v.is, &[1, 2, 3, 4, 5]);

    Ok(())
}

#[test]
fn long_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        ls: LongArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .long_array("ls", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(&*v.ls, &[1, 2, 3, 4, 5]);

    Ok(())
}

#[test]
fn long_array_cannot_be_deserialized_to_int_array() {
    #[derive(Deserialize)]
    struct V {
        _ls: IntArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .long_array("_ls", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_err());
}

#[test]
fn long_array_cannot_be_deserialized_to_byte_array() {
    #[derive(Deserialize)]
    struct V {
        _ls: ByteArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .long_array("_ls", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_err());
}

#[test]
fn int_array_cannot_be_deserialized_to_byte_array() {
    #[derive(Deserialize)]
    struct V {
        _ls: ByteArray,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int_array("_ls", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_err());
}

#[test]
fn byte_array_zero_copy() {
    #[derive(Deserialize)]
    struct V<'a> {
        #[serde(borrow)]
        data: borrow::ByteArray<'a>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte_array("data", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert!(v.data.iter().eq([1, 2, 3, 4, 5]));
}

#[test]
fn int_array_zero_copy() {
    #[derive(Deserialize)]
    struct V<'a> {
        #[serde(borrow)]
        data: borrow::IntArray<'a>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int_array("data", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert!(v.data.iter().eq([1, 2, 3, 4, 5]));
}

#[test]
fn long_array_zero_copy() {
    #[derive(Deserialize)]
    struct V<'a> {
        #[serde(borrow)]
        data: borrow::LongArray<'a>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .long_array("data", &[1, 2, 3, 4, 5])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();
    assert!(v.data.iter().eq([1, 2, 3, 4, 5]));
}

#[test]
fn array_subslice_doesnt_panic() {
    #[derive(Deserialize)]
    struct V<'a> {
        #[serde(borrow)]
        _data: borrow::LongArray<'a>,
    }

    let payload = Builder::new()
        .start_compound("")
        .long_array("_data", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .end_compound()
        .build();

    // cut off the data
    assert!(from_bytes::<V>(&payload[..20]).is_err());
}

#[test]
fn nice_error_if_deserialize_array_to_seq() {
    // Since the handling of NBT Arrays is a bit surprising, we want to make
    // that if someone tries to deserialize an Array into a serde seq (like a
    // Vec) rather than the dedicated types, we give a nice error message to them.
    #[derive(Deserialize)]
    struct V {
        _data: Vec<i64>,
    }

    let payload = Builder::new()
        .start_compound("")
        .long_array("_data", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .end_compound()
        .build();

    let res = from_bytes::<V>(&payload);
    match res {
        Ok(_) => panic!("expected err"),
        Err(e) => assert!(e.to_string().contains("Array")),
    }
}

#[test]
fn ints_to_bool() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        a: bool,
        b: bool,
        c: bool,
        d: bool,
    }

    let payload = Builder::new()
        .start_compound("")
        .byte("a", 123)
        .short("b", 1)
        .int("c", 2)
        .long("d", 0)
        .end_compound()
        .build();

    let v = from_all::<V>(&payload);
    assert_eq!(
        v,
        V {
            a: true,
            b: true,
            c: true,
            d: false
        }
    )
}

#[test]
fn direct_to_non_compound() {
    // Ensure that only compounds can be deserialized, not raw values inside
    // NBT.
    let payload = Builder::new().string_payload("some string").build();
    assert!(from_reader::<_, String>(&*payload).is_err());
}

#[test]
fn primitive_to_bytes() {
    // Ensure trying to deserialize a primitive to bytes fails.
    #[derive(Debug, Deserialize, PartialEq)]
    struct V<'a> {
        hello: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("")
        .int("hello", 123)
        .end_compound()
        .build();
    assert!(from_reader::<_, V>(&*payload).is_err());
}

#[test]
fn cannot_get_float_as_array() {
    #[derive(Deserialize)]
    struct V<'a> {
        _arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("_arr", Tag::Float, 2)
        .float_payload(1.)
        .float_payload(2.)
        .end_compound()
        .build();

    assert!(from_reader::<_, V>(&*payload).is_err());
}

#[test]
fn list_of_end_to_bytes() {
    #[derive(Deserialize)]
    struct V<'a> {
        _arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("_arr", Tag::End, 2)
        .tag(Tag::End)
        .tag(Tag::End)
        .end_compound()
        .build();

    assert!(from_reader::<_, V>(&*payload).is_err());
}

#[test]
fn empty_list_of_end_valid() {
    #[derive(Deserialize)]
    struct V {
        _arr: Vec<()>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("_arr", Tag::End, 0)
        .end_compound()
        .build();

    assert!(from_reader::<_, V>(&*payload).is_ok());
}

#[test]
fn nonempty_list_of_end_invalid() {
    #[derive(Deserialize)]
    struct V {
        _arr: Vec<()>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("_arr", Tag::End, 1)
        .tag(Tag::End)
        .end_compound()
        .build();

    assert!(from_reader::<_, V>(&*payload).is_err());
}

#[test]
fn long_list_invalid_with_option() {
    #[derive(Deserialize)]
    struct V {
        _arr: Vec<()>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("_arr", Tag::Byte, 2)
        .byte_payload(1)
        .byte_payload(2)
        .end_compound()
        .build();

    assert!(from_bytes_with_opts::<V>(&payload, DeOpts::new().max_seq_len(1)).is_err());
    assert!(from_bytes_with_opts::<V>(&payload, DeOpts::new().max_seq_len(2)).is_ok());
}

#[test]
fn untagged_enum_with_arrays() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum Array {
        Byte(ByteArray),
        Int(IntArray),
        Long(LongArray),
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        arr: Array,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int_array("arr", &[1, 2, 3])
        .end_compound()
        .build();

    let v: V = from_all(&payload);
    assert_eq!(v.arr, Array::Int(IntArray::new(vec![1, 2, 3])));
}

#[test]
fn tuple_struct_with_long_array() {
    #[derive(Debug, Deserialize, PartialEq)]
    pub struct PackedBits(pub LongArray);

    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        bits: PackedBits,
    }
    let payload = Builder::new()
        .start_compound("object")
        .long_array("bits", &[1, 2, 3])
        .end_compound()
        .build();

    let v: V = from_all(&payload);
    assert_eq!(v.bits, PackedBits(LongArray::new(vec![1, 2, 3])));
}

#[test]
fn negative_seq_lens() {
    let payload = Builder::new()
        .start_compound("")
        .start_list("list", Tag::Byte, -1)
        .raw_bytes(&[0; 1024])
        .end_compound()
        .build();

    assert!(from_bytes::<Value>(&payload).is_err());
}

#[test]
fn i128_from_int_array() {
    #[derive(Deserialize)]
    struct V {
        max: u128,
        min: i128,
        zero: i128,
        counting: u128,
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::IntArray)
        .name("max")
        .int_payload(4)
        // All bits are 1
        .int_array_payload(&[u32::MAX as i32; 4])
        .tag(Tag::IntArray)
        .name("min")
        .int_payload(4)
        // Only first bit is 1
        .int_array_payload(&[1 << 31, 0, 0, 0])
        .tag(Tag::IntArray)
        .name("zero")
        .int_payload(4)
        .int_array_payload(&[0; 4])
        .tag(Tag::IntArray)
        .name("counting")
        .int_payload(4)
        .int_array_payload(&[1, 2, 3, 4])
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.max, u128::MAX);
    assert_eq!(v.min, i128::MIN);
    assert_eq!(v.zero, 0);
    // Calculated with: 1 << 96 | 2 << 64 | 3 << 32 | 4
    assert_eq!(v.counting, 79228162551157825753847955460);
}

#[test]
fn i128_from_invalid_int_array() {
    #[derive(Deserialize)]
    struct V {
        _i: i128,
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::IntArray)
        .name("_i")
        .int_payload(3)
        .int_array_payload(&[1, 2, 3])
        .tag(Tag::End)
        .build();
    let v: Result<V> = from_bytes(payload.as_slice());
    assert!(v.is_err());

    // Although number of bytes is correct, won't be accepted
    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::ByteArray)
        .name("_i")
        .int_payload(16)
        .int_array_payload(&[1; 16])
        .tag(Tag::End)
        .build();
    let v: Result<V> = from_bytes(payload.as_slice());
    assert!(v.is_err());
}

#[test]
fn byte_array_value_from_reader() {
    let data = Builder::new()
        .start_compound("")
        .byte_array("test1", &[1, 2, 3])
        .int_array("test2", &[1, 2, 3])
        .long_array("test3", &[1, 2, 3])
        .end_compound()
        .build();

    let r = Cursor::new(data);
    // let r = &data;
    let _v: Value = from_reader(r).unwrap();
}

#[test]
fn networked_compound() {
    // TAG_Compound
    // {
    //     TAG_String("networked"): "compound"
    // }
    let data = b"\
        \x0a\
            \x08\
                \x00\x09networked\
                \x00\x08compound\
        \x00";

    let v: HashMap<String, Value> = from_bytes_with_opts(data, DeOpts::network_nbt()).unwrap();
    assert_eq!(v["networked"], "compound");
}

#[test]
fn nested_networked_compound() {
    // TAG_Compound
    // {
    //     TAG_Compound("nested")
    //     {
    //         TAG_String("networked"): "compound"
    //     }
    // }
    let data = b"\
        \x0a\
            \x0a\
            \x00\x06nested\
                \x08\
                    \x00\x09networked\
                    \x00\x08compound\
            \x00\
        \x00";

    let v: HashMap<String, HashMap<String, Value>> =
        from_bytes_with_opts(data, DeOpts::network_nbt()).unwrap();
    assert_eq!(&v["nested"]["networked"], "compound");
}

#[test]
fn error_nested_non_named_compounds() {
    // TAG_Compound
    // {
    //     TAG_Compound
    //     {
    //         TAG_String("double nested unnamed"): "compound"
    //     }
    // }
    let data = b"\
        \x0a\
            \x0a\
                \x08\
                    \x00\x15double nested unnamed\
                    \x00\x08compound\
            \x00\
        \x00";

    let v: Result<Value> = from_bytes_with_opts(data, DeOpts::network_nbt());
    assert!(v.is_err())
}
