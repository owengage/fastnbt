//! fastnbt aims for fast parsing of NBT data from *Minecraft: Java
//! Edition*. This format is used to store many things in Minecraft.
//!
//! A `serde` compatible deserializer can be found in the `de` module. This
//! deserialiser works on an in-memory `&[u8]`, meaning you need all of the NBT
//! data in memory. This has the advantage of allowing you to avoid memory
//! allocations in some cases. See the `de` module for more information.
//!
//! If you require processing a large amount of NBT data that you do not want to
//! keep in memory, or NBT data that you do not know the structure of, you can
//! use the `Parser`. This does not allow you to deserialize into Rust
//! `struct`s, but does allow low memory footprint processing on NBT data.
//!
//! Both this and related crates are under one [fastnbt Github
//! repository](https://github.com/owengage/fastnbt)

use num_enum::TryFromPrimitive;

pub mod de;
pub mod error;

mod bits;
pub use bits::*;

mod stream;
pub use stream::*;

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
