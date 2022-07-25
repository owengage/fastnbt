use serde::Deserialize;

use crate::{
    de2::{from_bytes, from_reader},
    test::builder::Builder,
    Tag,
};

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
fn simple_byte() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct V {
        abc: i8,
        def: i8,
    }

    let payload = Builder::new()
        .tag(Tag::Compound)
        .name("object")
        .tag(Tag::Byte)
        .name("abc")
        .byte_payload(123)
        .tag(Tag::Byte)
        .name("def")
        .byte_payload(111)
        .tag(Tag::End)
        .build();

    let v: V = from_all(payload.as_slice());

    assert_eq!(v.abc, 123);
    assert_eq!(v.def, 111);
}
