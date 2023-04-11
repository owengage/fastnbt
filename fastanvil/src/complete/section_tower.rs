use std::ops::Range;

use crate::complete::section::Section;
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
}

impl From<&java::SectionTower<java::Section>> for SectionTower {
    fn from(current_tower: &java::SectionTower<java::Section>) -> Self {
        let mut sections = vec![];

        for section in current_tower.sections() {
            sections.push(section.into())
        }

        SectionTower {
            sections,
            y_min: current_tower.y_min(),
            y_max: current_tower.y_max(),
        }
    }
}
