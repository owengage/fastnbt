use std::ops::Range;

use crate::biome::Biome;
use crate::complete::section_tower::SectionTower;
use crate::{dimension, Block, CurrentJavaChunk, HeightMode, JavaChunk};

pub struct Chunk {
    pub status: String,

    pub sections: SectionTower,
}

impl Chunk {
    pub fn from_bytes(data: &[u8]) -> fastnbt::error::Result<Self> {
        return match JavaChunk::from_bytes(data)? {
            JavaChunk::Post18(chunk) => Ok(chunk.into()),
            JavaChunk::Pre18(_) => {
                todo!()
            }
            JavaChunk::Pre13(_) => {
                todo!()
            }
        };
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = &Block> {
        self.sections.iter_blocks()
    }
}

impl dimension::Chunk for Chunk {
    fn status(&self) -> String {
        self.status.clone()
    }

    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize {
        todo!()
    }

    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome> {
        todo!()
    }

    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        self.sections.block(x, y, z)
    }

    fn y_range(&self) -> Range<isize> {
        self.sections.y_range()
    }
}

impl From<CurrentJavaChunk> for Chunk {
    fn from(current_java_chunk: CurrentJavaChunk) -> Self {
        Chunk {
            status: current_java_chunk.status.clone(),
            sections: current_java_chunk.sections.unwrap().into(),
        }
    }
}
