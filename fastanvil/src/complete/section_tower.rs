use std::ops::Range;

use crate::complete::section::{Section, SectionBlockIter};
use crate::{java, Block};

pub struct SectionTower {
    sections: Vec<Section>,

    y_min: isize,
    y_max: isize,
}

impl SectionTower {
    pub fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        if !self.y_range().contains(&y) || !(0..16).contains(&x) || !(0..16).contains(&z) {
            return None;
        }

        let section_index = self.y_to_index(y);

        let section = self.sections.get(section_index).unwrap();

        //first compute current section y then sub that from the ask y to get the y in the section
        let section_y = y - ((16 * section_index) as isize + self.y_min);

        section.block(x, section_y as usize, z)
    }

    fn y_to_index(&self, y: isize) -> usize {
        ((y - self.y_min) / 16) as usize
    }

    pub fn y_range(&self) -> Range<isize> {
        self.y_min..self.y_max
    }

    pub fn iter_blocks(&self) -> SectionTowerBlockIter {
        SectionTowerBlockIter::new(self)
    }
}

impl From<java::SectionTower<java::Section>> for SectionTower {
    fn from(current_tower: java::SectionTower<java::Section>) -> Self {
        let mut tower = SectionTower {
            sections: vec![],
            y_min: current_tower.y_min(),
            y_max: current_tower.y_max(),
        };

        for section in current_tower.take_sections() {
            tower.sections.push(section.into())
        }

        tower
    }
}

pub struct SectionTowerBlockIter<'a> {
    sections: &'a Vec<Section>,

    section_index_current: usize,
    section_iter_current: SectionBlockIter<'a>,
}

impl<'a> SectionTowerBlockIter<'a> {
    pub fn new(section_tower: &'a SectionTower) -> Self {
        Self {
            sections: &section_tower.sections,
            section_iter_current: section_tower.sections.get(0).unwrap().iter_blocks(),
            section_index_current: 0,
        }
    }
}

impl<'a> Iterator for SectionTowerBlockIter<'a> {
    type Item = &'a Block;

    fn next(&mut self) -> Option<Self::Item> {
        return match self.section_iter_current.next() {
            None => {
                //check if it was the last section
                if self.section_index_current >= self.sections.len() - 1 {
                    return None;
                }

                self.section_index_current += 1;
                self.section_iter_current = self
                    .sections
                    .get(self.section_index_current)
                    .unwrap()
                    .iter_blocks();

                self.section_iter_current.next()
            }
            Some(block) => Some(block),
        };
    }
}
