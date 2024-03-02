//! This module contains a serde serializer for NBT data. This should be able to
//! serialize most structures to NBT. Use [`to_bytes`][`crate::to_bytes`] or
//! [`to_writer`][`crate::to_writer`]. There are 'with opts' functions for more
//! serialization control.
//!
//! Some Rust structures have no sensible mapping to NBT data. These cases will
//! result in an error (not a panic). If you find a case where you think there
//! is a valid way to serialize it, please open an issue.
//!
//! The examples directory contains some examples. The [`de`][`crate::de`]
//! module contains more information about (de)serialization.
//!
//! # 128 bit integers and UUIDs
//!
//! UUIDs tend to be stored in NBT using 4-long IntArrays. When serializing
//! `i128` or `u128`, an IntArray of length 4 will be produced. This is stored
//! as big endian i.e. the most significant bit (and int) is first.
//!
//! # Root compound name
//!
//! A valid NBT compound must have a name, including the root compound. For most
//! Minecraft data this is simply the empty string. If you need to control the
//! name of this root compound you can use
//! [`to_bytes_with_opts`][`crate::to_bytes_with_opts`] and
//! [`to_writer_with_opts`][`crate::to_writer_with_opts`]. For example the
//! [unofficial schematic
//! format](https://minecraft.wiki/w/Schematic_file_format):
//!
//! ```no_run
//! use serde::{Serialize, Deserialize};
//! use fastnbt::{Value, ByteArray, SerOpts};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! #[serde(rename_all = "PascalCase")]
//! pub struct Schematic {
//!     blocks: ByteArray,
//!     data: ByteArray,
//!     tile_entities: Vec<Value>,
//!     entities: Vec<Value>,
//!     width: i16,
//!     height: i16,
//!     length: i16,
//!     materials: String,
//! }
//!
//! let structure = todo!(); // make schematic
//! let bytes = fastnbt::to_bytes_with_opts(&structure, SerOpts::new().root_name("Schematic")).unwrap();
//! ```
mod array_serializer;
mod name_serializer;
mod serializer;
mod write_nbt;

pub use serializer::*;
