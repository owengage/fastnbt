use serde::Deserialize;

use crate::borrow;
use crate::error::Result;
use crate::ByteArray;
use crate::IntArray;
use crate::LongArray;
use crate::{from_bytes, test::builder::Builder};

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
    assert!(v.bs.iter().eq(&[1, 2, 3, 4, 5]));

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

#[test]
fn array_subslice_doesnt_panic() {
    #[derive(Deserialize)]
    struct V<'a> {
        #[serde(borrow)]
        _data: borrow::LongArray<'a>,
    }

    let payload = Builder::new()
        .start_compound("")
        .long_array("_data", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .end_compound()
        .build();

    // cut off the data
    assert!(matches!(from_bytes::<V>(&payload[..20]), Err(_)));
}

#[test]
fn nice_error_if_deserialize_array_to_seq() {
    // Since the handling of NBT Arrays is a bit surprising, we want to make
    // that if someone tries to deserialize an Array into a serde seq (like a
    // Vec) rather than the dedicated types, we give a nice error message to them.
    #[derive(Deserialize)]
    struct V {
        _data: Vec<i64>,
    }

    let payload = Builder::new()
        .start_compound("")
        .long_array("_data", &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
        .end_compound()
        .build();

    let res = from_bytes::<V>(&payload);
    match res {
        Ok(_) => panic!("expected err"),
        Err(e) => assert!(e.to_string().contains("Array")),
    }
}
