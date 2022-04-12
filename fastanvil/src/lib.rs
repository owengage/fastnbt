//! For handling Minecraft's region format, Anvil.
//!
//! `anvil::Region` can be given a `Read` and `Seek` type eg a file in order to extract chunk data.

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
