//! This module contains a serde serializer for NBT data. This should be able to
//! serialize most structures to NBT. Use [`to_bytes`][`crate::to_bytes`] or
//! [`to_writer`][`crate::to_writer`].
//!
//! Some Rust structures have no sensible mapping to NBT data. These cases will
//! result in an error (not a panic). If you find a case where you think there
//! is a valid way to serialize it, please open an issue.
//!
//! The examples directory contains some examples.
mod array_serializer;
mod name_serializer;
mod serializer;
mod write_nbt;

pub use serializer::*;
