mod array_serializer;
mod name_serializer;
mod serializer;
mod write_nbt;

use serializer::*;
use std::vec;

use serde::Serialize;

use crate::error::Result;

pub fn to_bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    let mut result = vec![];
    let mut serializer = Serializer {
        writer: &mut result,
        state: State::Compound {
            current_field: String::new(),
        },
    };
    v.serialize(&mut serializer)?;
    Ok(result)
}
