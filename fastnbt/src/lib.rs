//! fastnbt aims for fast deserializing of NBT data from *Minecraft: Java
//! Edition*. This format is used by the game to store various things, such as
//! the world data and player inventories.
//!
//! * For documentation and examples of serde deserialization, see [`de`].
//! * For a `serde_json`-like `Value` type see [`Value`].
//! * For NBT array types see [`ByteArray`], [`IntArray`], and [`LongArray`].
//! * For 'zero-copy' NBT array types see [`borrow`].
//!
//! Both this and related crates are under one [fastnbt Github
//! repository](https://github.com/owengage/fastnbt)
//!
//! ```toml
//! [dependencies]
//! fastnbt = "1"
//! ```
//!
//! # Byte, Int and Long array types
//!
//! There are three array types in NBT. To capture these, use [`ByteArray`],
//! [`IntArray`], and [`LongArray`]. These NBT types do not deserialize straight
//! into serde sequences like `Vec` in order to preserve the information from
//! the original NBT. Without these types, it is not possible to tell if some
//! data came from a NBT List or an NBT Array.
//!
//! Use these in your own data structures. They all implement
//! [`Deref`][`std::ops::Deref`] for dereferencing into the underlying `Vec`.
//!
//! For versions that borrow their data, see [`borrow`]. For more information
//! about deserialization see [`de`].
//!
//! An example of deserializing a section of a chunk:
//!
//! ```no_run
//! use fastnbt::LongArray;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! #[serde(rename_all = "PascalCase")]
//! pub struct Section {
//!     pub block_states: Option<LongArray>,
//!     pub y: i8,
//! }
//!
//!# fn main(){
//!     let buf: &[u8] = unimplemented!("get a buffer from somewhere");
//!     let section: Section = fastnbt::de::from_bytes(buf).unwrap();
//!     let states = section.block_states.unwrap();
//!
//!     for long in states.iter() {
//!         // do something
//!     }
//! # }
//! ```
//!
//! # Example: Player inventory
//!
//! This example demonstrates printing out a players inventory and ender chest
//! contents from the [player dat
//! files](https://minecraft.gamepedia.com/Player.dat_format) found in worlds.
//!
//! Here we
//! * use serde's renaming attributes to have rustfmt conformant field names,
//! * use lifetimes to save on some string allocations (see [`de`] for more
//!   info), and
//! * use the `Value` type to deserialize a field we don't know the exact
//!   structure of.
//!
//!```no_run
//! use std::borrow::Cow;
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
//!     // We typically avoid allocating a string here.
//!     // See `fastnbt::de` docs for more info.
//!     id: Cow<'a, str>,
//!
//!     // Also get the less structured properties of the object.
//!     tag: Option<Value>,
//!
//!     // We need to rename fields a lot.
//!     #[serde(rename = "Count")]
//!     count: i8,
//! }
//!
//!# fn main() {
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
//!# }
//! ```
//!
//! # `Read` based parser
//!
//! A lower level parser also exists in the `stream` module that only requires
//! the `Read` trait on the input. This parser however doesn't support
//! deserializing to Rust objects directly.
//!

use serde::Deserialize;

pub mod borrow;
pub mod de;
pub mod error;
pub mod ser;
pub mod stream;

mod arrays;
mod de_arrays;
mod value;

pub use arrays::*;
pub use value::*;

#[cfg(test)]
mod test;

use std::{convert::TryFrom, fmt::Display};

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

// Crates exist to generate this code for us, but would add to our compile
// times, so we instead write it out manually, the tags will very rarely change
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

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Tag::End => "end",
            Tag::Byte => "byte",
            Tag::Short => "short",
            Tag::Int => "int",
            Tag::Long => "long",
            Tag::Float => "float",
            Tag::Double => "double",
            Tag::ByteArray => "byte-array",
            Tag::String => "string",
            Tag::List => "list",
            Tag::Compound => "compound",
            Tag::IntArray => "int-array",
            Tag::LongArray => "long-array",
        };
        f.write_str(s)
    }
}
