use std::ops::Range;

use once_cell::sync::Lazy;

pub use block::*;
pub use chunk::*;
use fastnbt::{error::Result, from_bytes};
pub use heightmaps::*;
pub use section::*;
pub use section_data::*;
pub use section_tower::*;

use crate::{biome::Biome, Chunk, HeightMode};

/// 1.2 to 1.12
pub mod pre13;
/// 1.13 to 1.17
pub mod pre18;

mod block;
mod chunk;
pub mod chunk_block_iter;
mod heightmaps;
mod section;
mod section_data;
mod section_tower;

pub static AIR: Lazy<Block> = Lazy::new(|| Block {
    name: "minecraft:air".to_owned(),
    encoded: "minecraft:air|".to_owned(),
    archetype: BlockArchetype::Airy,
});
pub static SNOW_BLOCK: Lazy<Block> = Lazy::new(|| Block {
    name: "minecraft:snow_block".to_owned(),
    encoded: "minecraft:snow_block|".to_owned(),
    archetype: BlockArchetype::Snowy,
});

/// A Minecraft chunk.
#[derive(Debug)]
pub enum JavaChunk {
    Post18(CurrentJavaChunk),
    Pre18(pre18::JavaChunk),
    Pre13(pre13::JavaChunk),
}

impl JavaChunk {
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let chunk: Result<CurrentJavaChunk> = from_bytes(data);

        match chunk {
            Ok(chunk) => Ok(Self::Post18(chunk)),
            Err(_) => match from_bytes::<pre18::JavaChunk>(data) {
                Ok(chunk) => Ok(Self::Pre18(chunk)),
                Err(_) => Ok(Self::Pre13(from_bytes::<pre13::JavaChunk>(data)?)),
            },
        }
    }
}

// TODO: Find a better way to dispatch these methods.
impl Chunk for JavaChunk {
    fn status(&self) -> String {
        match self {
            JavaChunk::Post18(c) => c.status(),
            JavaChunk::Pre18(c) => c.status(),
            JavaChunk::Pre13(c) => c.status(),
        }
    }

    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize {
        match self {
            JavaChunk::Post18(c) => c.surface_height(x, z, mode),
            JavaChunk::Pre18(c) => c.surface_height(x, z, mode),
            JavaChunk::Pre13(c) => c.surface_height(x, z, mode),
        }
    }

    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome> {
        match self {
            JavaChunk::Post18(c) => c.biome(x, y, z),
            JavaChunk::Pre18(c) => c.biome(x, y, z),
            JavaChunk::Pre13(c) => c.biome(x, y, z),
        }
    }

    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        match self {
            JavaChunk::Post18(c) => c.block(x, y, z),
            JavaChunk::Pre18(c) => c.block(x, y, z),
            JavaChunk::Pre13(c) => c.block(x, y, z),
        }
    }

    fn y_range(&self) -> Range<isize> {
        match self {
            JavaChunk::Post18(c) => c.y_range(),
            JavaChunk::Pre18(c) => c.y_range(),
            JavaChunk::Pre13(c) => c.y_range(),
        }
    }
}