use std::ops::Range;

use crate::biome::Biome;
use crate::complete::section_tower::SectionTower;
use crate::{dimension, Block, CurrentJavaChunk, HeightMode};

pub struct Chunk {
    pub status: String,

    pub sections: SectionTower,
}

impl Chunk {
    pub fn from_current_chunk(current_java_chunk: &CurrentJavaChunk) -> Self {
        Chunk {
            status: current_java_chunk.status.clone(),
            sections: SectionTower::from_current_chunk(
                current_java_chunk.sections.as_ref().unwrap(),
            ),
        }
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
