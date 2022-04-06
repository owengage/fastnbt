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
    InsufficientData,
    IO(std::io::Error),
    InvalidOffset(isize, isize),
    InvalidChunkMeta,
    ChunkNotFound,
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
            Error::InsufficientData => f.write_str("insufficient data to parse chunk metadata"),
            Error::IO(e) => f.write_fmt(format_args!("io error: {:?}", e)),
            Error::InvalidOffset(x, z) => {
                f.write_fmt(format_args!("invalid offset: x = {}, z = {}", x, z))
            }
            Error::InvalidChunkMeta => {
                f.write_str("compression scheme was not recognised for chunk")
            }
            Error::ChunkNotFound => f.write_str("chunk not found in region"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
use std::io::Cursor;
#[cfg(test)]
pub struct Builder {
    inner: Vec<u8>,
}

#[cfg(test)]
impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl Builder {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn location(mut self, offset: u32, sectors: u8) -> Self {
        self.inner.extend_from_slice(&offset.to_be_bytes()[1..4]);
        self.inner.push(sectors);
        self
    }

    pub fn build(mut self) -> Cursor<Vec<u8>> {
        let padded_sector_count = (self.inner.len() / SECTOR_SIZE) + 1;
        self.inner.resize(padded_sector_count * SECTOR_SIZE, 0);
        Cursor::new(self.inner)
    }

    pub fn build_unpadded(self) -> Cursor<Vec<u8>> {
        Cursor::new(self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_offset() {
        let r = Builder::new().location(2, 1).build();
        let r = RegionBuffer::new(r);
        match r.chunk_location(32, 32) {
            Err(Error::InvalidOffset(32, 32)) => {}
            _ => panic!("should error"),
        }
    }

    #[test]
    fn invalid_offset_only_in_x() {
        let r = Builder::new().location(2, 1).build();
        let r = RegionBuffer::new(r);
        match r.chunk_location(32, 0) {
            Err(Error::InvalidOffset(32, 0)) => {}
            _ => panic!("should error"),
        }
    }

    #[test]
    fn invalid_offset_only_in_z() {
        let r = Builder::new().location(2, 1).build();
        let r = RegionBuffer::new(r);
        match r.chunk_location(0, 32) {
            Err(Error::InvalidOffset(0, 32)) => {}
            _ => panic!("should error"),
        }
    }

    #[test]
    fn offset_beyond_data_given() {
        let r = Builder::new().location(2, 1).build_unpadded();
        let r = RegionBuffer::new(r);
        match r.chunk_location(1, 0) {
            Err(Error::IO(inner)) if inner.kind() == std::io::ErrorKind::UnexpectedEof => {}
            o => panic!("should error {:?}", o),
        }
    }
    #[test]
    fn first_location() -> Result<()> {
        let r = Builder::new().location(2, 1).build();
        let r = RegionBuffer::new(r);

        assert_eq!(
            ChunkLocation {
                begin_sector: 2,
                sector_count: 1,
                x: 0,
                z: 0
            },
            r.chunk_location(0, 0)?
        );
        Ok(())
    }
}
