use std::ops::Range;
use std::slice::Iter;

use crate::biome::Biome;
use crate::{java, Block, StatesIter};

//could remove duplication though another layer similar to BlockData<> | BiomData<>
pub struct Section {
    block_palette: Vec<Block>,

    //This will increase in x, then z, then y.
    //u16 should be enough for all blocks
    blocks: Option<Vec<u16>>,

    biome_palette: Vec<Biome>,

    //This will increase in x, then z, then y.
    //u8 should be enough for all bioms
    biomes: Option<Vec<u8>>,
}

impl Section {
    pub fn block(&self, x: usize, y: usize, z: usize) -> Option<&Block> {
        return match &self.blocks {
            None => Some(self.block_palette.get(0).unwrap()),
            Some(blocks) => {
                let index = y * (16 * 16) + z * 16 + x;

                self.block_palette.get(*blocks.get(index).unwrap() as usize)
            }
        };
    }

    pub fn biome(&self, x: usize, y: usize, z: usize) -> Option<Biome> {
        let x = x / 4;
        let y = y / 4;
        let z = z / 4;

        return match &self.biomes {
            None => Some(self.biome_palette.get(0).unwrap().clone()),
            Some(bioms) => {
                let index = y * (4 * 4) + z * 4 + x;

                Some(
                    self.biome_palette
                        .get(*bioms.get(index).unwrap() as usize)
                        .unwrap()
                        .clone(),
                )
            }
        };
    }

    pub fn iter_blocks(&self) -> SectionBlockIter {
        SectionBlockIter::new(self)
    }
}

impl From<java::Section> for Section {
    fn from(current_section: java::Section) -> Self {
        let blocks = match current_section.block_states.try_iter_indices() {
            None => None,
            Some(block_iter) => Some(block_iter.map(|index| index as u16).collect()),
        };

        let biomes = match current_section.biomes.try_iter_indices() {
            None => None,
            Some(biome_iter) => Some(biome_iter.map(|index| index as u8).collect()),
        };

        Section {
            block_palette: Vec::from(current_section.block_states.palette()),
            blocks,
            biome_palette: Vec::from(current_section.biomes.palette()),
            biomes,
        }
    }
}

pub struct SectionBlockIter<'a> {
    section: &'a Section,

    block_index_iter: Option<Iter<'a, u16>>,
    default_block_iter: Option<Range<i32>>,
}

impl<'a> SectionBlockIter<'a> {
    pub fn new(section: &'a Section) -> Self {
        let mut iter = Self {
            section,
            block_index_iter: None,
            default_block_iter: None,
        };

        match &section.blocks {
            None => iter.default_block_iter = Some(0..(16 * 16 * 16)),
            Some(blocks) => iter.block_index_iter = Some(blocks.iter()),
        }

        iter
    }
}

impl<'a> Iterator for SectionBlockIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        return if let Some(iter) = self.default_block_iter.as_mut() {
            match iter.next() {
                None => None,
                Some(_) => self.section.block_palette.get(0),
            }
        } else {
            match self.block_index_iter.as_mut().unwrap().next() {
                None => None,
                Some(index) => self.section.block_palette.get(*index as usize),
            }
        };
    }
}
