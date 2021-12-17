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
fn simple_short() {
    let v = Single { val: 1000u16 };
    let bs = to_bytes(&v).unwrap();
    let expected = Builder::new()
        .start_compound("")
        .short("val", 1000)
        .end_compound()
        .build();

    assert_eq!(expected, bs);
}
