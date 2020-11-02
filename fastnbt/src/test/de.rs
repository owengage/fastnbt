use std::collections::HashMap;

use crate::de::from_bytes;
use crate::error::{Error, Result};
use crate::Tag;

use super::builder::Builder;
use serde::Deserialize;

#[test]
fn simple_byte() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        abc: i8,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("abc")
        .byte_payload(123)
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.abc, 123);
    Ok(())
}

#[test]
fn simple_short_to_i16() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        abc: i16,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Short)
        .name("abc")
        .short_payload(256)
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.abc, 256);
    Ok(())
}

#[test]
fn simple_short_to_u16() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        abc: u16,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Short)
        .name("abc")
        .short_payload(256)
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.abc, 256);
    Ok(())
}

#[test]
fn negative_short_to_u16_errors() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        _abc: u16,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Short)
        .name("_abc")
        .short_payload(-1)
        .tag(Tag::End)
        .build();

    let v: Result<V> = from_bytes(payload.as_slice());

    assert!(v.is_err());
    Ok(())
}

#[test]
fn multiple_fields() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        a: u8,
        b: u16,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("a")
        .byte_payload(123)
        .tag(Tag::Short)
        .name("b")
        .short_payload(1024)
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
    assert_eq!(v.b, 1024);
    Ok(())
}

#[test]
fn numbers_into_u32() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        a: u32,
        b: u32,
        c: u32,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("a")
        .byte_payload(123)
        .tag(Tag::Short)
        .name("b")
        .short_payload(2 << 8)
        .tag(Tag::Int)
        .name("c")
        .int_payload(2 << 24)
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
    assert_eq!(v.b, 2 << 8);
    assert_eq!(v.c, 2 << 24);
    Ok(())
}

#[test]
fn string_into_ref_str() -> Result<()> {
    #[derive(Deserialize)]
    struct V<'a> {
        a: &'a str,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::String)
        .name("a")
        .string_payload("hello")
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!("hello", v.a);

    Ok(())
}

#[test]
fn string_into_string() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        a: String,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::String)
        .name("a")
        .string_payload("hello")
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!("hello", v.a);

    Ok(())
}

#[test]
fn nested_compound() -> Result<()> {
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
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("a")
        .byte_payload(123)
        .tag(Tag::Compound)
        .name("nested")
        .tag(Tag::Byte)
        .name("b")
        .byte_payload(1)
        .tag(Tag::End)
        .tag(Tag::End)
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
    assert_eq!(v.nested.b, 1);
    Ok(())
}

#[test]
fn unwanted_byte() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        a: u32,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("a")
        .byte_payload(123)
        .tag(Tag::Byte)
        .name("b")
        .byte_payload(1)
        .tag(Tag::End)
        .build();

    // requires impl of deserialize_ignored_any
    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, 123);
    Ok(())
}

#[test]
fn unwanted_primative_payloads() -> Result<()> {
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
    Ok(())
}

#[test]
fn simple_list() -> Result<()> {
    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.a, [1, 2, 3]);
    Ok(())
}

#[test]
fn list_of_compounds() -> Result<()> {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Inner {
        a: u32,
    }

    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(
        v.inner,
        Some(vec![Inner { a: 1 }, Inner { a: 2 }, Inner { a: 3 }])
    );
    assert_eq!(v.after, 123);
    Ok(())
}

#[test]
fn complex_nesting() -> Result<()> {
    #[derive(Deserialize, PartialEq, Debug)]
    struct Inner {
        a: u32,
        b: Option<Vec<i32>>,
    }

    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice()).unwrap();

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
    Ok(())
}

#[test]
fn optional() -> Result<()> {
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
    Ok(())
}

#[test]
fn unit_just_requires_presense() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        _unit: (),
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte("_unit", 0)
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_ok());
    Ok(())
}

#[test]
fn unit_not_present_errors() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        _unit: (),
    }

    let payload = Builder::new()
        .start_compound("object")
        .end_compound()
        .build();

    assert!(from_bytes::<V>(payload.as_slice()).is_err());
    Ok(())
}

#[test]
fn ignore_compound() -> Result<()> {
    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.a, 123);

    Ok(())
}

#[test]
fn ignore_primitives_in_ignored_compound() -> Result<()> {
    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.a, 123);

    Ok(())
}

#[test]
fn ignore_list() -> Result<()> {
    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.a, 123);

    Ok(())
}

#[test]
fn ignore_list_of_compound() -> Result<()> {
    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.a, 123);

    Ok(())
}

#[test]
fn byte_array_from_list_bytes() -> Result<()> {
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [1, 2, 3]);

    Ok(())
}

#[test]
fn byte_array_from_nbt_byte_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::ByteArray)
        .name("arr")
        .int_payload(3)
        .byte_array_payload(&[1, 2, 3])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [1, 2, 3]);

    Ok(())
}

#[test]
fn byte_array_from_nbt_int_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::IntArray)
        .name("arr")
        .int_payload(3)
        .int_array_payload(&[1, 2, 3])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3]);

    Ok(())
}

#[test]
fn byte_array_from_nbt_long_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V<'a> {
        arr: &'a [u8],
    }

    let payload = Builder::new()
        .start_compound("object")
        .tag(Tag::LongArray)
        .name("arr")
        .int_payload(2)
        .long_array_payload(&[1, 2])
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);

    Ok(())
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
fn byte_array_from_nbt_int_list() -> Result<()> {
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [0, 0, 0, 1, 0, 0, 0, 2]);

    Ok(())
}

#[test]
fn byte_array_from_nbt_long_list() -> Result<()> {
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.arr, [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2]);

    Ok(())
}

#[test]
fn newtype_struct() -> Result<()> {
    #[derive(Deserialize)]
    struct Inner(u8);

    #[derive(Deserialize)]
    struct V {
        a: Inner,
    }

    let payload = Builder::new()
        .start_compound("object")
        .byte("a", 123)
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.a.0, 123);

    Ok(())
}

#[test]
fn vec_from_nbt_byte_array() -> Result<()> {
    #[derive(Deserialize)]
    struct V {
        a: Vec<u8>,
        b: Vec<u32>,
        c: Vec<u64>,
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.a, [1, 2, 3]);
    assert_eq!(v.b, [4, 5, 6]);
    assert_eq!(v.c, [7, 8, 9]);
    Ok(())
}

#[test]
fn unit_variant_enum() -> Result<()> {
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(v.e1, E::A);
    assert_eq!(v.e2, E::B);
    assert_eq!(v.e3, E::C);
    Ok(())
}

#[derive(Deserialize)]
struct Blockstates<'a>(&'a [u8]);

#[test]
fn blockstates() -> Result<()> {
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

    let v: V = from_bytes(payload.as_slice())?;
    assert_eq!(
        [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3],
        v.states.0
    );

    Ok(())
}

#[test]
fn ignore_integral_arrays() -> Result<()> {
    #[derive(Deserialize)]
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

    assert!(from_bytes::<V>(payload.as_slice()).is_ok());
    Ok(())
}

#[test]
fn fixed_array() -> Result<()> {
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

    let v: Level = from_bytes(payload.as_slice())?;
    assert_eq!([1, 2, 3], v.inner[0].a);
    assert_eq!([4, 5, 6], v.inner[1].a);
    assert_eq!([7, 8, 9], v.inner[2].a);
    Ok(())
}

#[test]
fn type_mismatch_string() -> Result<()> {
    #[derive(Deserialize, Debug)]
    pub struct V {
        a: String,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int("a", 123)
        .end_compound() // end of outer compound
        .build();

    let res = from_bytes::<V>(payload.as_slice());

    assert!(res.is_err());
    assert_eq!(Error::TypeMismatch(Tag::Int, "string"), res.unwrap_err());

    Ok(())
}

#[test]
fn type_mismatch_f32_from_int() -> Result<()> {
    #[derive(Deserialize, Debug)]
    pub struct V {
        a: f32,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int("a", 123)
        .end_compound() // end of outer compound
        .build();

    let res = from_bytes::<V>(payload.as_slice());

    assert!(res.is_err());
    assert_eq!(
        Error::TypeMismatch(Tag::Int, "float/double"),
        res.unwrap_err()
    );

    Ok(())
}

#[test]
fn type_mismatch_f64_from_int() -> Result<()> {
    #[derive(Deserialize, Debug)]
    pub struct V {
        a: f64,
    }

    let payload = Builder::new()
        .start_compound("object")
        .int("a", 123)
        .end_compound() // end of outer compound
        .build();

    let res = from_bytes::<V>(payload.as_slice());

    assert!(res.is_err());
    assert_eq!(
        Error::TypeMismatch(Tag::Int, "float/double"),
        res.unwrap_err()
    );

    Ok(())
}

#[test]
fn type_mismatch_int_from_str() -> Result<()> {
    #[derive(Deserialize, Debug)]
    pub struct V {
        a: i32,
    }

    let payload = Builder::new()
        .start_compound("object")
        .string("a", "hello")
        .end_compound() // end of outer compound
        .build();

    let res = from_bytes::<V>(payload.as_slice());

    assert!(res.is_err());
    assert_eq!(
        Error::TypeMismatch(Tag::String, "integral"),
        res.unwrap_err()
    );

    Ok(())
}

#[test]
fn basic_palette_item() -> Result<()> {
    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    pub struct PaletteItem {
        name: String,
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

    let res: PaletteItem = from_bytes(payload.as_slice())?;

    assert_eq!(res.name, "minecraft:redstone_ore");

    Ok(())
}
