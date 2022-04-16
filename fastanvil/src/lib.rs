//! For handling Minecraft's region format, Anvil. This crate is mostly to
//! support creating maps of Minecraft worlds and is not stable (per-1.0). The
//! [`Region`] struct is probably the most generally useful part in this crate.
//!
//! This crate also contains a [`JavaChunk`] that allows deserializing 1.18
//! down to about 1.15 chunks into some structs. This doesn't record all
//! information from a chunk however, eg entities are lost. It is not suitable
//! for serializing back into a region.
//!
//! You can create your own chunk structures to (de)serialize using [`fastnbt`].
//!
//! [`Region`] can be given a `Read`, `Write` and `Seek` type eg a file in
//! order to read and write chunk data.

pub mod biome;
pub mod tex;

mod bits;
mod dimension;
mod files;
mod java;
mod region;
mod render;
mod rendered_palette;

pub use bits::*;
pub use dimension::*;
pub use files::*;
pub use java::*;
pub use region::*;
pub use render::*;
pub use rendered_palette::*;

#[cfg(test)]
mod test;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    InvalidOffset(isize, isize),
    UnknownCompression(u8),
    ChunkTooLarge,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IO(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IO(e) => f.write_fmt(format_args!("io error: {e:?}")),
            Error::InvalidOffset(x, z) => {
                f.write_fmt(format_args!("invalid offset: x = {x}, z = {z}"))
            }
            Error::UnknownCompression(scheme) => f.write_fmt(format_args!(
                "compression scheme ({scheme}) was not recognised for chunk"
            )),
            Error::ChunkTooLarge => f.write_str("chunk too large to store"),
        }
    }
}

impl std::error::Error for Error {}
