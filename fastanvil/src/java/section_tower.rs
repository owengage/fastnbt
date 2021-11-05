use serde::Deserialize;

use crate::SectionLike;

/// SectionTower represents the set of sections that make up a Minecraft chunk.
/// It has a custom deserialization in order to more efficiently lay out the
/// sections for quick access.
#[derive(Debug)]
pub struct SectionTower<S> {
    sections: Vec<S>,
    map: Vec<Option<usize>>,
    y_min: isize,
    y_max: isize,
}

impl<S> SectionTower<S> {
    pub fn get_section_for_y(&self, y: isize) -> Option<&S> {
        if y >= self.y_max || y < self.y_min {
            // TODO: This occurs a lot in hermitcraft season 7. Probably some
            // form of bug?
            return None;
        }

        let lookup_index = y_to_index(y, self.y_min);

        let section_index = *self.map.get(lookup_index as usize)?;
        self.sections.get(section_index?)
    }

    pub fn y_min(&self) -> isize {
        self.y_min
    }

    pub fn y_max(&self) -> isize {
        self.y_max
    }
}

impl<'de, S: SectionLike + Deserialize<'de>> Deserialize<'de> for SectionTower<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let sections: Vec<S> = Deserialize::deserialize(deserializer)?;
        if sections.is_empty() {
            return Ok(Self {
                sections,
                map: vec![],
                y_min: 0,
                y_max: 0,
            });
        }

        // We need to figure out how deep the world goes. Since 1.17 the depth
        // of worlds can be customized. Each section in the chunk will have a
        // 'y' value. We need to find the minimum value here, and that will tell
        // us the minimum world y.
        let lowest_section = sections
            .iter()
            .min_by_key(|s| s.y())
            .expect("checked no empty above");

        // Sometimes the lowest section is a 'null' section. This isn't actually
        // part of the world, it just indicates there no more sections below.
        // You can tell if it's 'null terminated' by the palette and
        // blockstates.
        let null_term = lowest_section.is_terminator();

        let min = if null_term {
            lowest_section.y() as isize + 1
        } else {
            lowest_section.y() as isize
        };
        let max = sections
            .iter()
            .max_by_key(|s| s.y())
            .map(|s| s.y())
            .unwrap() as isize;

        let mut sparse_sections = vec![None; (1 + max - min) as usize];

        for (i, sec) in sections.iter().enumerate() {
            // Don't bother adding the null section.
            if sec.y() == lowest_section.y() && null_term {
                continue;
            }

            let sec_index = (sec.y() as isize - min) as usize;

            sparse_sections[sec_index] = Some(i);
        }

        Ok(Self {
            sections,
            map: sparse_sections,
            y_min: 16 * min,
            y_max: 16 * (max + 1),
        })
    }
}

const fn y_to_index(y: isize, y_min: isize) -> u8 {
    ((y - y_min) >> 4) as u8
}
