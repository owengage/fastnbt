use serde::Deserialize;

use crate::borrow;
use crate::error::Result;
use crate::ByteArray;
use crate::IntArray;
use crate::LongArray;
use crate::{de::from_bytes, test::builder::Builder};

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
    assert_eq!(&*v.bs, &[1, 2, 3, 4, 5]);

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

    assert!(matches!(from_bytes::<V>(payload.as_slice()), Err(_)));
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

    assert!(matches!(from_bytes::<V>(payload.as_slice()), Err(_)));
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

    assert!(matches!(from_bytes::<V>(payload.as_slice()), Err(_)));
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
