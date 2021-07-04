use std::ops::Deref;

use serde::Deserialize;

use crate::error::Result;
use crate::ByteArray;
use crate::{de::from_bytes, test::builder::Builder};

#[test]
fn byte_array_from_nbt_byte_array() -> Result<()> {
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

// TODO: Zero copy stuff
