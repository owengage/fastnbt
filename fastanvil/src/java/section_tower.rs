use serde::Deserialize;

use crate::{Section, MAX_Y, MIN_Y};

/// SectionTower represents the set of sections that make up a Minecraft chunk.
/// It has a custom deserialization in order to more efficiently lay out the
/// sections for quick access.
#[derive(Debug)]
pub struct SectionTower {
    sections: Vec<Section>,
    map: [Option<usize>; 24],
}

impl SectionTower {
    pub fn get_section_for_y(&self, y: isize) -> Option<&Section> {
        if self.sections.is_empty() {
            return None;
        }

        if y >= MAX_Y || y < MIN_Y {
            // TODO: This occurs a lot in hermitcraft season 7. Probably some
            // form of bug?
            return None;
        }

        // Need to be careful. y = -5 should return -1. If we did normal integer
        // division -5/16 would give us 0.
        let containing_section_y = ((y as f64) / 16.0).floor() as i8;

        let section_index = self.map[(containing_section_y + 4) as usize]?;

        self.sections.get(section_index)
    }
}

impl<'de> Deserialize<'de> for SectionTower {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let sections: Vec<Section> = Deserialize::deserialize(deserializer)?;
        let mut map: [Option<usize>; 24] = Default::default();

        for (i, sec) in sections.iter().enumerate() {
            map[(sec.y + 4) as usize] = Some(i);
        }

        Ok(Self { sections, map })
    }
}
