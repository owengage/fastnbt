use super::super::de::from_bytes;
use super::super::error::{Error, Result};
use super::super::Tag;

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
        abc: u16,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Short)
        .name("abc")
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
        inner: Vec<Inner>,
    }

    let payload = Builder::new()
        .start_compound("object")
        .start_list("inner", Tag::Compound, 3)
        .byte("a", 1)
        .end_compound()
        .byte("a", 2)
        .end_compound()
        .byte("a", 3)
        .end_compound()
        .end_compound()
        .build();

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.inner, [Inner { a: 1 }, Inner { a: 2 }, Inner { a: 3 }]);
    Ok(())
}
