use serde::Deserialize;

use crate::{de2::from_bytes, test::builder::Builder, Tag};

#[test]
fn simple_byte() {
    #[derive(Deserialize)]
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

    let v: V = from_bytes(payload.as_slice()).unwrap();

    assert_eq!(v.abc, 123);
    assert_eq!(v.def, 111);
}
