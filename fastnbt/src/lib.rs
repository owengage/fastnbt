//! fastnbt aims for fast parsing of NBT data from *Minecraft: Java Edition*.
//! This format is used by the game to store various things, such as the world
//! data and player inventories.
//!
//! For documentation and examples of serde deserialization, see the
//! [`de`](de/index.html) module.
//!
//! Both this and related crates are under one [fastnbt Github
//! repository](https://github.com/owengage/fastnbt)
//!
//! ```toml
//! [dependencies]
//! fastnbt = "0.18"
//! ```
//!
//! # Quick example
//!
//! This example demonstrates printing out a players inventory and ender chest
//! contents from the [player dat
//! files](https://minecraft.gamepedia.com/Player.dat_format) found in worlds.
//! We leverage serde's renaming attribute to have rustfmt conformant field
//! names, use lifetimes to save on some string allocations, and use the `Value`
//! type to deserialize a field we don't specify the exact structure of.
//!
//!```no_run
//! use fastnbt::error::Result;
//! use fastnbt::{de::from_bytes, Value};
//! use flate2::read::GzDecoder;
//! use serde::Deserialize;
//! use std::io::Read;
//!
//! #[derive(Deserialize, Debug)]
//! #[serde(rename_all = "PascalCase")]
//! struct PlayerDat<'a> {
//!     data_version: i32,
//!
//!     #[serde(borrow)]
//!     inventory: Vec<InventorySlot<'a>>,
//!     ender_items: Vec<InventorySlot<'a>>,
//! }
//!
//! #[derive(Deserialize, Debug)]
//! struct InventorySlot<'a> {
//!     id: &'a str,        // We avoid allocating a string here.
//!     tag: Option<Value>, // Also get the less structured properties of the object.
//!
//!     // We need to rename fields a lot.
//!     #[serde(rename = "Count")]
//!     count: i8,
//! }
//!
//! fn main() {
//!     let args: Vec<_> = std::env::args().skip(1).collect();
//!     let file = std::fs::File::open(args[0].clone()).unwrap();
//!
//!     // Player dat files are compressed with GZip.
//!     let mut decoder = GzDecoder::new(file);
//!     let mut data = vec![];
//!     decoder.read_to_end(&mut data).unwrap();
//!
//!     let player: Result<PlayerDat> = from_bytes(data.as_slice());
//!
//!     println!("{:#?}", player);
//! }
//! ```
//!
//! # `Read` based parser
//!
//! A lower level parser also exists in the `stream` module that only requires
//! the `Read` trait on the input. This parser however doesn't support
//! deserializing to Rust objects directly.
//!

use serde::Deserialize;

pub mod de;
pub mod error;
pub mod stream;

mod arrays;
pub use arrays::*;

pub(crate) mod de_arrays;

use std::{collections::HashMap, convert::TryFrom};

/// An NBT tag. This does not carry the value or the name of the data.
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum Tag {
    /// Represents the end of a Compound object.
    End = 0,
    /// Equivalent to i8.
    Byte = 1,
    /// Equivalent to i16.
    Short = 2,
    /// Equivalent to i32.
    Int = 3,
    /// Equivalent to i64
    Long = 4,
    /// Equivalent to f32.
    Float = 5,
    /// Equivalent to f64.
    Double = 6,
    /// Represents as array of Byte (i8).
    ByteArray = 7,
    /// Represents a Unicode string.
    String = 8,
    /// Represents a list of other objects, elements are not required to be the same type.
    List = 9,
    /// Represents a struct-like structure.
    Compound = 10,
    /// Represents as array of Int (i32).
    IntArray = 11,
    /// Represents as array of Long (i64).
    LongArray = 12,
}

/// Value is a complete NBT value. It owns it's data. The Byte, Short, Int and
/// Long NBT types are all deserialized into `i64`. Compounds and Lists are
/// resursively deserialized.
///
/// ```no_run
/// # use fastnbt::Value;
/// # use fastnbt::error::Result;
/// # use std::collections::HashMap;
/// #
/// # fn main() -> Result<()> {
/// #   let mut buf = vec![];
///     let compound: HashMap<String, Value> = fastnbt::de::from_bytes(buf.as_slice())?;
///     match compound["DataVersion"] {
///         Value::Integral(ver) => println!("Version: {}", ver),
///         _ => {},
///     }
///     println!("{:#?}", compound);
/// #   Ok(())
/// # }
/// ```
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Value {
    /// Any integral value, ie a byte, short, int and long all deserialize to
    /// this type. This simplifies both usage and implementation. If you care
    /// about the exact integral type you may need to write a custom
    /// `Deserialise` type with serde. Please also open an issue with your use
    /// case!
    Integral(i64),

    /// A double. serde distinguishes between f32 and f64, so we do too.
    Double(f64),

    /// A float. serde distinguishes between f32 and f64, so we do too.
    Float(f32),

    /// An array of i64. This will either have been a ByteArray, IntArray or
    /// LongArray in the original NBT.
    IntegralArray(Vec<i64>),

    /// A unicode string.
    String(String),

    /// A List of NBT values. Each value may have a different structure/type.
    List(Vec<Value>),

    /// A compound, which is a struct-like object.
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

impl From<Tag> for u8 {
    fn from(tag: Tag) -> Self {
        match tag {
            Tag::End => 0,
            Tag::Byte => 1,
            Tag::Short => 2,
            Tag::Int => 3,
            Tag::Long => 4,
            Tag::Float => 5,
            Tag::Double => 6,
            Tag::ByteArray => 7,
            Tag::String => 8,
            Tag::List => 9,
            Tag::Compound => 10,
            Tag::IntArray => 11,
            Tag::LongArray => 12,
        }
    }
}

#[cfg(test)]
mod test;
