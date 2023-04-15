use std::ops::Range;
use std::sync::RwLock;

use serde::Deserialize;

use crate::{biome::Biome, Block, Chunk, HeightMode};
use crate::{expand_heightmap, Heightmaps, Section, SectionTower};

use super::AIR;

impl Chunk for CurrentJavaChunk {
    fn status(&self) -> String {
        self.status.clone()
    }

    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize {
        let mut heightmap = self.lazy_heightmap.read().unwrap();
        if heightmap.is_none() {
            drop(heightmap);
            self.recalculate_heightmap(mode);
            heightmap = self.lazy_heightmap.read().unwrap();
        }
        heightmap.unwrap()[z * 16 + x] as isize
    }

    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome> {
        // Going to be awkward because the biomes are now paletted, and so are
        // the string not a number.

        let sections = self.sections.as_ref()?;
        let sec = sections.get_section_for_y(y)?;
        let sec_y = (y - sec.y as isize * 16) as usize;

        sec.biomes.at(x, sec_y, z).cloned()
    }

    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        let sections = self.sections.as_ref()?;
        let sec = sections.get_section_for_y(y)?;
        let sec_y = (y - sec.y as isize * 16) as usize;

        Some(sec.block_states.at(x, sec_y, z).unwrap_or(&AIR))
    }

    fn y_range(&self) -> Range<isize> {
        match &self.sections {
            Some(sections) => Range {
                start: sections.y_min(),
                end: sections.y_max(),
            },
            None => Range { start: 0, end: 0 },
        }
    }
}

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
pub struct CurrentJavaChunk {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,

    // Maybe put section and heightmaps together and serde flatten?
    pub sections: Option<SectionTower<Section>>,

    #[serde(rename = "Heightmaps")]
    pub heightmaps: Option<Heightmaps>,

    #[serde(rename = "Status")]
    pub status: String,

    #[serde(skip)]
    pub(crate) lazy_heightmap: RwLock<Option<[i16; 256]>>,
}

impl CurrentJavaChunk {
    pub fn recalculate_heightmap(&self, mode: HeightMode) {
        // TODO: Find top section and start there, pointless checking 320 down
        // if its a 1.16 chunk.

        let mut map = [0; 256];

        match mode {
            HeightMode::Trust => {
                let updated = self
                    .heightmaps
                    .as_ref()
                    .and_then(|hm| hm.motion_blocking.as_ref())
                    .map(|hm| {
                        let y_min = self.sections.as_ref().unwrap().y_min();
                        expand_heightmap(hm, y_min, self.data_version)
                    })
                    .map(|hm| map.copy_from_slice(hm.as_slice()))
                    .is_some();

                if updated {
                    *self.lazy_heightmap.write().unwrap() = Some(map);
                    return;
                }
            }
            HeightMode::Calculate => {} // fall through to calc mode
        }

        let y_range = self.y_range();
        let y_end = y_range.end;

        for z in 0..16 {
            for x in 0..16 {
                // start at top until we hit a non-air block.
                for i in y_range.clone() {
                    let y = y_end - i;
                    let block = self.block(x, y - 1, z);

                    if block.is_none() {
                        continue;
                    }

                    if !["minecraft:air", "minecraft:cave_air"]
                        .as_ref()
                        .contains(&block.unwrap().name())
                    {
                        map[z * 16 + x] = y as i16;
                        break;
                    }
                }
            }
        }

        *self.lazy_heightmap.write().unwrap() = Some(map);
    }
}
