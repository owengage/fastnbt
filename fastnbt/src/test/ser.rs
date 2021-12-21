use crate::ser::to_bytes;
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
fn simple_integral() {
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
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}
