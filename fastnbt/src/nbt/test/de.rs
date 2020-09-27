// use crate::nbt::de::from_reader;
// use crate::nbt::error::{Error, Result};

// use super::*;
// use crate::nbt::test::builder::Builder;
// use serde::Deserialize;

// #[test]
// fn empty_payload() {
//     let payload = Builder::new().build();
//     let v: Result<()> = from_reader(payload.as_slice());
// }

// #[test]
// fn simple_byte() -> Result<()> {
//     #[derive(Deserialize)]
//     struct V {
//         abc: i8,
//     }

//     let payload = Builder::new()
//         .tag(Tag::Compound)
//         .name("object")
//         .tag(Tag::Byte)
//         .name("abc")
//         .byte_payload(123)
//         .tag(Tag::End)
//         .build();

//     let v: V = from_reader(payload.as_slice()).unwrap();

//     assert_eq!(v.abc, 123);
//     Ok(())
// }
