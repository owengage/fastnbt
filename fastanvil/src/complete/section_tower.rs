use std::ops::Range;

use crate::complete::section::Section;
use crate::{java, Block};

pub struct SectionTower {
    sections: Vec<Section>,

    y_min: isize,
    y_max: isize,
}

impl SectionTower {
    pub fn from_current_chunk(current_tower: &java::SectionTower<java::Section>) -> Self {
        let mut sections = vec![];

        for section in current_tower.sections() {
            sections.push(Section::from_current_section(section))
        }

        SectionTower {
            sections: sections,
            y_min: current_tower.y_min(),
            y_max: current_tower.y_max(),
        }
    }

    pub fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
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
