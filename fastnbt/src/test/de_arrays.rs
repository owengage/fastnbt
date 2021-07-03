use serde::Deserialize;

use crate::{de::from_bytes, test::builder::Builder};

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
