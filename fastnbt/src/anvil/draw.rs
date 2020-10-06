//! functionality for drawing maps.
use super::*;
use crate::nbt;
use types::Chunk;

pub trait RegionDrawer {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk);
}

pub struct RegionMap<T> {
    pub data: Vec<T>,
    pub x_region: isize,
    pub z_region: isize,
}

impl<T: Clone> RegionMap<T> {
    pub fn new(x_region: isize, z_region: isize, default: T) -> Self {
        let mut data: Vec<T> = Vec::new();
        data.resize(16 * 16 * 32 * 32, default);
        Self {
            data,
            x_region,
            z_region,
        }
    }

    pub fn chunk_mut(&mut self, x: usize, z: usize) -> &mut [T] {
        let len = 16 * 16;
        let begin = (x * 32 + z) * len; // TODO: x or z ordered?
        &mut self.data[begin..begin + len]
    }

    pub fn chunk(&self, x: usize, z: usize) -> &[T] {
        let len = 16 * 16;
        let begin = (x * 32 + z) * len; // TODO: x or z ordered?
        &self.data[begin..begin + len]
    }
}

pub type Rgb = [u8; 3];

#[derive(Debug)]
pub enum DrawError {
    ParseAnvil(super::Error),
    ParseNbt(nbt::error::Error),
    IO(std::io::Error),
    MissingHeightMap,
    InvalidPalette,
}

impl From<super::Error> for DrawError {
    fn from(err: super::Error) -> DrawError {
        DrawError::ParseAnvil(err)
    }
}

impl From<crate::nbt::error::Error> for DrawError {
    fn from(err: crate::nbt::error::Error) -> DrawError {
        DrawError::ParseNbt(err)
    }
}

impl From<std::io::Error> for DrawError {
    fn from(err: std::io::Error) -> Self {
        DrawError::IO(err)
    }
}

pub type DrawResult<T> = std::result::Result<T, DrawError>;

pub fn parse_region<F: RegionDrawer + ?Sized>(
    mut region: Region<std::fs::File>,
    draw_to: &mut F,
) -> DrawResult<()> {
    let closure = |x: usize, z: usize, buf: &Vec<u8>| {
        let chunk: DrawResult<types::Chunk> = Ok(nbt::de::from_bytes(buf.as_slice()).unwrap());
        match chunk {
            Ok(mut chunk) => draw_to.draw(x, z, &mut chunk),
            Err(DrawError::MissingHeightMap) => {} // skip this chunk.
            Err(e) => panic!(e),
        }
    };

    region.for_each_chunk(closure)?;
    Ok(())
}
