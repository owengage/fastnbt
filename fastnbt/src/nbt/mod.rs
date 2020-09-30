use num_enum::TryFromPrimitive;

/// Serde deserialisation.
pub mod de;

/// Serde error and result type.
pub mod error;

/// Allows streaming of NBT data without prior knowledge of the structure.
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
