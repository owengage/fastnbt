//! fastnbt aims for fast parsing of NBT data from *Minecraft: Java Edition*.
//! This format is used to store many things in Minecraft.
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

//use num_enum::TryFromPrimitive;
use serde::Deserialize;

pub mod de;
pub mod error;
pub mod stream;

use std::{collections::HashMap, convert::TryFrom};

/// An NBT tag. This does not carry the value or the name.
#[derive(Debug, PartialEq, Clone, Copy)]
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

/// Value is a complete NBT value. It owns it's data. The Byte, Short, Int and
/// Long NBT types are all deserialized into `i64`. Compounds and Lists are
/// resursively deserialised.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Value {
    Integral(i64),
    Double(f64),
    Float(f32),
    IntegralArray(Vec<i64>),
    String(String),
    List(Vec<Value>),
    Compound(HashMap<String, Value>),
}

// Crates exist to generate this code for us, but would add to our compile
// times, so we instead right it out manually, the tags will very rarely change
// so isn't a massive burden, but saves a significant amount of compile time.
impl TryFrom<u8> for Tag {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        use Tag::*;
        Ok(match value {
            0 => End,
            1 => Byte,
            2 => Short,
            3 => Int,
            4 => Long,
            5 => Float,
            6 => Double,
            7 => ByteArray,
            8 => String,
            9 => List,
            10 => Compound,
            11 => IntArray,
            12 => LongArray,
            13..=u8::MAX => return Err(()),
        })
    }
}

#[cfg(test)]
mod test;
