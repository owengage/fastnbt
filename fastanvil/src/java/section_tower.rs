use lazy_static::lazy_static;
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

const MAP_SIZE: usize = (MAX_Y - MIN_Y) as usize;

lazy_static! {
    // map every possible y value to the equivalent section y. This operation
    // is needed a lot while rendering, so we make it fast by having it be a
    // simple look up in a static array.
    static ref MAP: [u8; MAP_SIZE] = y_to_index_map();
}

impl SectionTower {
    pub fn get_section_for_y(&self, y: isize) -> Option<&Section> {
        if y >= MAX_Y || y < MIN_Y {
            // TODO: This occurs a lot in hermitcraft season 7. Probably some
            // form of bug?
            return None;
        }

        let lookup_index = MAP[(y - MIN_Y) as usize];
        let section_index = self.map[lookup_index as usize]?;
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

fn y_to_index_map() -> [u8; MAP_SIZE] {
    let mut map = [0u8; MAP_SIZE];

    for y in MIN_Y..MAX_Y {
        // Need to be careful. y = -5 should return -1. If we did normal integer
        // division -5/16 would give us 0.
        map[(y - MIN_Y) as usize] = (((y as f64) / 16.0).floor() as i8 + 4) as u8;
    }

    map
}
