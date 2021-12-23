use crate::{ser::to_bytes, Tag};
use serde::Serialize;

use super::builder::Builder;

#[derive(Serialize)]
struct Single<T: Serialize> {
    val: T,
}

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

// TODO: Test raw values fail serialization.
