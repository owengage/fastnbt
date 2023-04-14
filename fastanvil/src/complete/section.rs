use std::ops::Range;
use std::slice::Iter;

use crate::biome::Biome;
use crate::pre18::Pre18Section;
use crate::{java, Block, AIR};

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
            Some(biomes) => {
                let index = y * (4 * 4) + z * 4 + x;

                Some(
                    self.biome_palette
                        .get(*biomes.get(index).unwrap() as usize)
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

impl From<(Pre18Section, &[Biome])> for Section {
    fn from((current_section, current_biomes): (Pre18Section, &[Biome])) -> Self {
        let blocks;
        let block_pallet;

        match current_section.block_states {
            None => {
                block_pallet = vec![AIR.clone()];
                blocks = None
            }
            Some(block_states) => {
                block_pallet = current_section.palette;

                blocks = Some(
                    block_states
                        .iter_indices(block_pallet.len())
                        .map(|index| index as u16)
                        .collect(),
                );
            }
        }

        let (biome_palette, biomes) = create_biome_palette(current_biomes);

        Section {
            block_palette: Vec::from(block_pallet),
            blocks,
            biome_palette,
            biomes,
        }
    }
}

impl From<(&[Block], &[Biome])> for Section {
    fn from((current_blocks, current_biomes): (&[Block], &[Biome])) -> Self {
        let mut blocks = vec![];
        let mut block_palette = vec![];

        for block in current_blocks {
            match block_palette
                .iter()
                .position(|check_block: &Block| check_block.name() == block.name())
            {
                None => {
                    block_palette.push(block.clone());
                    blocks.push((block_palette.len() - 1) as u16)
                }
                Some(index) => blocks.push(index as u16),
            }
        }

        let mut blocks = Some(blocks);

        //optimize if possible
        if block_palette.len() == 1 {
            blocks = None;
        }

        let (biome_palette, biomes) = create_biome_palette(current_biomes);

        Section {
            block_palette,
            blocks,
            biome_palette,
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

//todo generic so it an also be used for blocks
fn create_biome_palette(all_biomes: &[Biome]) -> (Vec<Biome>, Option<Vec<u8>>) {
    let mut biomes = vec![];
    let mut biome_palette = vec![];

    for biome in all_biomes {
        match biome_palette
            .iter()
            .position(|check_biome| check_biome == biome)
        {
            None => {
                biome_palette.push(biome.clone());
                biomes.push((biome_palette.len() - 1) as u8)
            }
            Some(index) => biomes.push(index as u8),
        }
    }

    let mut biomes = Some(biomes);

    //optimize if possible
    if biome_palette.len() == 1 {
        biomes = None;
    }

    (biome_palette, biomes)
}
