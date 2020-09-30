use super::*;
use crate::nbt::{self};
use crate::nbt2;
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

// pub struct RegionHeightmapDrawer<'a> {
//     map: &'a mut RegionMap<Rgb>,
// }

// impl<'a> RegionHeightmapDrawer<'a> {
//     pub fn new(map: &'a mut RegionMap<Rgb>) -> Self {
//         Self { map }
//     }
// }

// impl<'a> RegionDrawer for RegionHeightmapDrawer<'a> {
//     fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &Chunk) {
//         let data = (*self.map).chunk_mut(xc_rel, zc_rel);

//         for z in 0..16 {
//             for x in 0..16 {
//                 const SEA_LEVEL: u16 = 63;
//                 let height = chunk.heights[x * 16 + z];

//                 if height <= SEA_LEVEL {
//                     data[x * 16 + z] = [height as u8, height as u8, 150];
//                 } else {
//                     let height = (height - 63) * 2;
//                     data[x * 16 + z] = [height as u8, 150, height as u8];
//                 }
//             }
//         }
//     }
// }

#[derive(Debug)]
pub enum DrawError {
    ParseAnvil(super::Error),
    ParseNbt(nbt::Error),
    ParseNbt2(nbt2::error::Error),
    IO(std::io::Error),
    MissingHeightMap,
    InvalidPalette,
}

impl From<nbt::Error> for DrawError {
    fn from(err: nbt::Error) -> DrawError {
        DrawError::ParseNbt(err)
    }
}

impl From<super::Error> for DrawError {
    fn from(err: super::Error) -> DrawError {
        DrawError::ParseAnvil(err)
    }
}

impl From<crate::nbt2::error::Error> for DrawError {
    fn from(err: crate::nbt2::error::Error) -> DrawError {
        DrawError::ParseNbt2(err)
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
        let chunk: DrawResult<types::Chunk> = Ok(nbt2::de::from_bytes(buf.as_slice()).unwrap());
        match chunk {
            Ok(mut chunk) => draw_to.draw(x, z, &mut chunk),
            Err(DrawError::MissingHeightMap) => {} // skip this chunk.
            Err(e) => panic!(e),
        }
    };

    region.for_each_chunk(closure)?;
    Ok(())
}
