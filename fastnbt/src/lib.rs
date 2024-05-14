//! fastnbt aims for fast deserializing and serializing of NBT data from
//! *Minecraft: Java Edition*. This format is used by the game to store various
//! things, such as the world data and player inventories.
//!
//! * For documentation and examples of serde (de)serialization, see [`ser`] and
//!   [`de`].
//! * For a `serde_json`-like `Value` type see [`Value`].
//! * To easily create values, see the [`nbt`] macro.
//! * For NBT array types see [`ByteArray`], [`IntArray`], and [`LongArray`].
//! * For zero-copy NBT array types see [`borrow`].
//!
//! Both this and related crates are under one [fastnbt Github
//! repository](https://github.com/owengage/fastnbt).
//!
//! ```toml
//! [dependencies]
//! fastnbt = "2"
//! ```
//!
//! # Byte, Int and Long array types
//!
//! The NBT format has 4 sequence types: lists, byte arrays, int arrays and long
//! arrays. To preserve the distinction between NBT lists and arrays, NBT array
//! data cannot be (de)serialized into sequences like `Vec`. To capture arrays,
//! use [`ByteArray`], [`IntArray`], and [`LongArray`]. An actual NBT list can
//! be captured by a `Vec` or other suitable container.
//!
//! Use these in your own data structures. They all implement
//! [`Deref`][`std::ops::Deref`] for dereferencing into a slice.
//!
//! For versions that borrow their data, see [`borrow`].
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
//! let buf: &[u8] = unimplemented!("get a buffer from somewhere");
//! let section: Section = fastnbt::from_bytes(buf).unwrap();
//! let states = section.block_states.unwrap();
//!
//! for long in states.iter() {
//!     // do something
//! }
//! # }
//! ```
//!
//! # Example: Player inventory
//!
//! This example demonstrates printing out a players inventory and ender chest
//! contents from the [player dat
//! files](https://minecraft.wiki/w/Player.dat_format) found in worlds.
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
//! use fastnbt::{from_bytes, Value};
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
//! # Stream based parser
//!
//! A lower level parser also exists in the [`stream`] module for use cases not
//! requiring deserialization into Rust objects. You can use [`from_reader`] for
//! full deserialization.
//!

use ser::Serializer;
use serde::{de as serde_de, Deserialize, Serialize};

pub mod borrow;
pub mod de;
pub mod error;
pub mod ser;
pub mod stream;
pub mod value;

mod arrays;
mod input;
#[macro_use]
mod macros;

pub use arrays::*;
pub use value::{from_value, to_value, Value};

#[cfg(test)]
mod test;

use crate::{
    de::Deserializer,
    error::{Error, Result},
};
use std::{
    convert::TryFrom,
    fmt::Display,
    io::{Read, Write},
};

/// An NBT tag. This does not carry the value or the name of the data.
#[derive(Deserialize, Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "arbitrary1", derive(arbitrary::Arbitrary))]
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

    fn try_from(value: u8) -> std::result::Result<Self, ()> {
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
    #[inline(always)]
    fn from(tag: Tag) -> Self {
        tag as u8
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

/// Serialize some `T` into NBT data. See the [`ser`] module for more
/// information.
pub fn to_bytes<T: Serialize>(v: &T) -> Result<Vec<u8>> {
    to_bytes_with_opts(v, Default::default())
}

/// Serialize some `T` into NBT data. See the [`ser`] module for more
/// information.
pub fn to_writer<T: Serialize, W: Write>(writer: W, v: &T) -> Result<()> {
    to_writer_with_opts(writer, v, Default::default())
}

/// Options for customizing serialization.
#[derive(Clone)]
pub struct SerOpts {
    root_name: String,
    /// Whether to include the root compound name.
    serialize_root_name: bool,
}

impl Default for SerOpts {
    fn default() -> Self {
        Self {
            root_name: Default::default(),
            serialize_root_name: true,
        }
    }
}

impl SerOpts {
    /// Create new options. This object follows a builder pattern.
    pub fn new() -> Self {
        Default::default()
    }

    pub fn network_nbt() -> Self {
        Self::new().serialize_root_compound_name(false)
    }

    pub fn serialize_root_compound_name(mut self, serialize_root_name: bool) -> Self {
        self.serialize_root_name = serialize_root_name;
        self
    }

    /// Set the root name (top level) of the compound. In most Minecraft data
    /// structures this is the empty string. The [`ser`][`crate::ser`] module
    /// contains an example.
    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self.serialize_root_name = true;
        self
    }
}

/// Serialize some `T` into NBT data. See the [`ser`] module for more
/// information. The options allow you to set things like the root name of the
/// compound when serialized.
pub fn to_bytes_with_opts<T: Serialize>(v: &T, opts: SerOpts) -> Result<Vec<u8>> {
    let mut result = vec![];
    let mut serializer = Serializer {
        writer: &mut result,
        root_name: opts.root_name,
        serialize_root_name: opts.serialize_root_name,
    };
    v.serialize(&mut serializer)?;
    Ok(result)
}

/// Serialize some `T` into NBT data. See the [`ser`] module for more
/// information. The options allow you to set things like the root name of the
/// compound when serialized.
pub fn to_writer_with_opts<T: Serialize, W: Write>(writer: W, v: &T, opts: SerOpts) -> Result<()> {
    let mut serializer = Serializer {
        writer,
        root_name: opts.root_name,
        serialize_root_name: opts.serialize_root_name,
    };
    v.serialize(&mut serializer)?;
    Ok(())
}

/// Deserialize into a `T` from some NBT data. See the [`de`] module for more
/// information.
///
/// ```no_run
/// # use fastnbt::Value;
/// # use flate2::read::GzDecoder;
/// # use std::io;
/// # use std::io::Read;
/// # use fastnbt::error::Result;
/// # fn main() -> Result<()> {
/// # let some_reader = io::stdin();
/// let mut decoder = GzDecoder::new(some_reader);
/// let mut buf = vec![];
/// decoder.read_to_end(&mut buf).unwrap();
///
/// let val: Value = fastnbt::from_bytes(buf.as_slice())?;
/// # Ok(())
/// # }
/// ```
pub fn from_bytes<'a, T>(input: &'a [u8]) -> Result<T>
where
    T: serde_de::Deserialize<'a>,
{
    from_bytes_with_opts(input, Default::default())
}

/// Deserialize into a `T` from some NBT data. See the [`de`] module for more
/// information.
///
/// ```no_run
/// # use fastnbt::Value;
/// # use flate2::read::GzDecoder;
/// # use std::io;
/// # use std::io::Read;
/// # use fastnbt::error::Result;
/// # fn main() -> Result<()> {
/// # use fastnbt::DeOpts;
/// # let some_reader = io::stdin();
/// let mut decoder = GzDecoder::new(some_reader);
/// let opts = DeOpts::network_nbt();
/// let val: Value = fastnbt::from_reader_with_opts(decoder, opts)?;
/// # Ok(())
/// # }
/// ```
pub fn from_reader_with_opts<'de, R, T>(reader: R, opts: DeOpts) -> Result<T>
    where
        T: serde_de::Deserialize<'de>,
        R: Read,
{
    let mut deserializer = Deserializer::from_reader(reader, opts);
    serde_de::Deserialize::deserialize(&mut deserializer)
}

/// Deserialize into a `T` from some NBT data. See the [`de`] module for more
/// information.
///
/// ```no_run
/// # use fastnbt::Value;
/// # use flate2::read::GzDecoder;
/// # use std::io;
/// # use std::io::Read;
/// # use fastnbt::error::Result;
/// # fn main() -> Result<()> {
/// # let some_reader = io::stdin();
/// let mut decoder = GzDecoder::new(some_reader);
/// let val: Value = fastnbt::from_reader(decoder)?;
/// # Ok(())
/// # }
/// ```
pub fn from_reader<'de, R, T>(reader: R) -> Result<T>
    where
        T: serde_de::Deserialize<'de>,
        R: Read,
{
    from_reader_with_opts(reader, Default::default())
}

/// Options for customizing deserialization.
#[derive(Clone)]
pub struct DeOpts {
    /// Maximum number of bytes a list or array can be.
    max_seq_len: usize,
    /// Whether compound tag names are expected to exist or not.
    expect_coumpound_names: bool,
}

impl DeOpts {
    /// Create new options. This object follows a builder pattern.
    pub fn new() -> Self {
        Default::default()
    }

    /// Creates a decoder for "network NBT" mode.
    /// At the moment that consists compound tags not having names (or their length).
    pub fn network_nbt() -> Self {
        Self::new().expect_coumpound_names(false)
    }

    /// Set the maximum length any given sequence can be, eg lists. This does
    /// not apply to NBT array types. This can help prevent panics on malformed
    /// data.
    pub fn max_seq_len(mut self, value: usize) -> Self {
        self.max_seq_len = value;
        self
    }

    /// Sets wheather the deserializer should expect compound tags to have names.
    pub fn expect_coumpound_names(mut self, value: bool) -> Self {
        self.expect_coumpound_names = value;
        self
    }
}

impl Default for DeOpts {
    fn default() -> Self {
        Self {
            max_seq_len: 10_000_000, // arbitrary high limit.
            expect_coumpound_names: true,
        }
    }
}

/// Similar to [`from_bytes`] but with options.
pub fn from_bytes_with_opts<'a, T>(input: &'a [u8], opts: DeOpts) -> Result<T>
where
    T: serde_de::Deserialize<'a>,
{
    const GZIP_MAGIC_BYTES: [u8; 2] = [0x1f, 0x8b];

    // Provide freindly error for the common case of passing GZip data to
    // `from_bytes`. This would be invalid starting data for NBT anyway.
    if input.starts_with(&GZIP_MAGIC_BYTES) {
        return Err(Error::bespoke(
            "from_bytes expects raw NBT, but input appears to be gzipped".to_string(),
        ));
    }

    let mut des = Deserializer::from_bytes(input, opts);
    let t = T::deserialize(&mut des)?;
    Ok(t)
}
