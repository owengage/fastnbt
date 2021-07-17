use std::{cell::RefCell, convert::TryFrom, ops::Range};

use fastnbt::{IntArray, LongArray};
use lazy_static::lazy_static;

use serde::Deserialize;

use crate::{expand_heightmap, Chunk, HeightMode};

use super::biome::Biome;

mod block;
mod blockstates;
mod section_tower;

pub use block::*;
pub use blockstates::*;
pub use section_tower::*;

lazy_static! {
    pub static ref AIR: Block = Block {
        name: "minecraft:air".to_owned(),
        encoded: "minecraft:air|".to_owned(),
        snowy: false,
        properties: Default::default(),
    };
    pub static ref SNOW_BLOCK: Block = Block {
        name: "minecraft:snow_block".to_owned(),
        encoded: "minecraft:snow_block|".to_owned(),
        snowy: true,
        properties: Default::default(),
    };
}

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JavaChunk {
    pub data_version: i32,
    pub level: Level,
}

impl Chunk for JavaChunk {
    fn status(&self) -> String {
        self.level.status.clone()
    }

    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize {
        if self.level.lazy_heightmap.borrow().is_none() {
            self.recalculate_heightmap(mode);
        }

        self.level.lazy_heightmap.borrow().unwrap()[z * 16 + x] as isize
    }

    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome> {
        let biomes = self.level.biomes.as_ref()?;

        // After 1.15 Each biome in i32, biomes split into 4-wide cubes, so
        // 4x4x4 per section.

        // v1.15 was only x/z, i32 per column.
        const V1_15: usize = 16 * 16;

        match biomes.len() {
            V1_15 => {
                // 1x1 columns stored z then x.
                let i = z * 16 + x;
                let biome = biomes[i];
                Biome::try_from(biome).ok()
            }
            _ => {
                // Assume latest
                let range = self.y_range();
                let y_shifted = (y.clamp(range.start, range.end - 1) - range.start) as usize;
                let i = (z / 4) * 4 + (x / 4) + (y_shifted / 4) * 16;

                let biome = *biomes.get(i)?;
                Biome::try_from(biome).ok()
            }
        }
    }

    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        let sec = self.level.sections.as_ref()?.get_section_for_y(y)?;

        // If a section is entirely air, then the block states are missing
        // entirely, presumably to save space.
        match &sec.block_states {
            None => Some(&AIR),
            Some(blockstates) => {
                let sec_y = (y - sec.y as isize * 16) as usize;
                let pal_index = blockstates.state(x, sec_y, z, sec.palette.len());
                sec.palette.get(pal_index)
            }
        }
    }

    fn y_range(&self) -> std::ops::Range<isize> {
        match &self.level.sections {
            Some(sections) => Range {
                start: sections.y_min(),
                end: sections.y_max(),
            },
            None => Range { start: 0, end: 0 },
        }
    }
}

/// A level describes the contents of the chunk in the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Level {
    #[serde(rename = "xPos")]
    pub x_pos: i32,

    #[serde(rename = "zPos")]
    pub z_pos: i32,

    pub biomes: Option<IntArray>,

    /// Can be empty if the chunk hasn't been generated properly yet.
    pub sections: Option<SectionTower>,

    pub heightmaps: Option<Heightmaps>,

    // Status of the chunk. Typically anything except 'full' means the chunk
    // hasn't been fully generated yet. We use this to skip chunks on map edges
    // that haven't been fully generated yet.
    pub status: String,

    #[serde(skip)]
    lazy_heightmap: RefCell<Option<[i16; 256]>>,
}

/// Various heightmaps kept up to date by Minecraft.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Heightmaps {
    pub motion_blocking: Option<LongArray>,
    //pub motion_blocking_no_leaves: Option<Heightmap>,
    //pub ocean_floor: Option<Heightmap>,
    //pub world_surface: Option<Heightmap>,
}

/// A vertical section of a chunk (ie a 16x16x16 block cube)
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub y: i8,

    pub block_states: Option<Blockstates>,

    #[serde(default)]
    pub palette: Vec<Block>,
}

impl JavaChunk {
    pub fn recalculate_heightmap(&self, mode: HeightMode) {
        // TODO: Find top section and start there, pointless checking 320 down
        // if its a 1.16 chunk.

        let mut map = [0; 256];

        match mode {
            HeightMode::Trust => {
                let updated = self
                    .level
                    .heightmaps
                    .as_ref()
                    .and_then(|hm| hm.motion_blocking.as_ref())
                    .map(|hm| {
                        // unwrap, if heightmaps exists, sections should... 🤞
                        let y_min = self.level.sections.as_ref().unwrap().y_min();
                        expand_heightmap(hm.as_slice(), y_min, self.data_version)
                    })
                    .map(|hm| map.copy_from_slice(hm.as_slice()))
                    .is_some();

                if updated {
                    self.level.lazy_heightmap.replace(Some(map));
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

        self.level.lazy_heightmap.replace(Some(map));
    }
}
