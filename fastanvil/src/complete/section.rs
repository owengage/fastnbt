use std::ops::Range;
use std::slice::Iter;

use crate::{java, Block};

pub struct Section {
    block_palette: Vec<Block>,

    //could be [] because of fix size
    //This will increase in x, then z, then y.
    blocks: Option<Vec<u16>>,
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

    pub fn iter_blocks(&self) -> SectionBlockIter {
        SectionBlockIter::new(self)
    }
}

impl From<java::Section> for Section {
    fn from(current_section: java::Section) -> Self {
        let mut blocks = None;

        if let Some(iter) = current_section.block_states.try_iter_indices() {
            let mut blocks_indexes = vec![];

            for block_index in iter {
                blocks_indexes.push(block_index as u16);
            }

            blocks = Some(blocks_indexes);
        }

        Section {
            block_palette: Vec::from(current_section.block_states.palette()),
            blocks,
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
