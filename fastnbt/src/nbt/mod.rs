//! For handling NBT data, which Minecraft uses for most data storage.
//!
//! `de` contains a standard Serde deserializer to let you deserialize NBT into structs.
//!
//! `stream` contains a parser to let you manually parse NBT, rather than putting it into a `struct`.
//! This can let you for example simply dump a bunch of NBT without knowing the size or structure.

use num_enum::TryFromPrimitive;

pub mod de;
pub mod error;
pub mod stream;

/// The NBT tag. This does not carry the value or the name.
#[derive(Debug, TryFromPrimitive, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Tag {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

#[cfg(test)]
mod test;
