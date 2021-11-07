use std::ops::Range;

use lazy_static::lazy_static;

mod block;
mod chunk;
mod heightmaps;
mod pre18;
mod section;
mod section_data;
mod section_tower;

pub use block::*;
pub use chunk::*;
pub use heightmaps::*;
pub use section::*;
pub use section_data::*;
pub use section_tower::*;
use serde::Deserialize;

use crate::{biome::Biome, Chunk, HeightMode};

lazy_static! {
    pub static ref AIR: Block = Block {
        name: "minecraft:air".to_owned(),
        encoded: "minecraft:air|".to_owned(),
        snowy: false,
        properties: Default::default(),
    };
    pub static ref SNOW_BLOCK: Block = Block {
        name: "minecraft:snow_block".to_owned(),
        encoded: "minecraft:snow_block|".to_owned(),
        snowy: true,
        properties: Default::default(),
    };
}

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JavaChunk {
    Post18(CurrentJavaChunk),
    Pre18(pre18::JavaChunk),
}

// TODO: Find a better way to dispatch these methods.
impl Chunk for JavaChunk {
    fn status(&self) -> String {
        match self {
            JavaChunk::Post18(c) => c.status(),
            JavaChunk::Pre18(c) => c.status(),
        }
    }

    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize {
        match self {
            JavaChunk::Post18(c) => c.surface_height(x, z, mode),
            JavaChunk::Pre18(c) => c.surface_height(x, z, mode),
        }
    }

    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome> {
        match self {
            JavaChunk::Post18(c) => c.biome(x, y, z),
            JavaChunk::Pre18(c) => c.biome(x, y, z),
        }
    }

    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        match self {
            JavaChunk::Post18(c) => c.block(x, y, z),
            JavaChunk::Pre18(c) => c.block(x, y, z),
        }
    }

    fn y_range(&self) -> Range<isize> {
        match self {
            JavaChunk::Post18(c) => c.y_range(),
            JavaChunk::Pre18(c) => c.y_range(),
        }
    }
}
