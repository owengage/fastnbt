use std::{cell::RefCell, convert::TryFrom};

use lazy_static::lazy_static;

use serde::Deserialize;

use crate::{expand_heightmap, Chunk, HeightMode, MAX_Y, MIN_Y};

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

        // Each biome in i32, biomes split into 4-wide cubes, so 4x4x4 per
        // section. 384 world height (320 + 64), so 384/16 subchunks.
        const V1_17: usize = 4 * 4 * 4 * 384 / 16;

        // Each biome in i32, biomes split into 4-wide cubes, so 4x4x4 per
        // section. 256 world height, so 256/16 subchunks.
        const V1_16: usize = 4 * 4 * 4 * 256 / 16;

        // v1.15 was only x/z, i32 per column.
        const V1_15: usize = 16 * 16;

        match biomes.len() {
            V1_16 | V1_17 => {
                //let after1_17 = self.data_version >= 2695;
                //let y_shifted = if after1_17 { y + 64 } else { y } as usize;

                // TODO: work out why y can be 0. Hole in the world?
                let y_shifted = y.max(0) as usize;
                let i = (z / 4) * 4 + (x / 4) + (y_shifted / 4) * 16;

                let biome = biomes[i];
                Biome::try_from(biome).ok()
            }
            V1_15 => {
                // 1x1 columns stored z then x.
                let i = z * 16 + x;
                let biome = biomes[i];
                Biome::try_from(biome).ok()
            }
            _ => None,
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
}

/// A level describes the contents of the chunk in the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Level {
    #[serde(rename = "xPos")]
    pub x_pos: i32,

    #[serde(rename = "zPos")]
    pub z_pos: i32,

    pub biomes: Option<Vec<i32>>,

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
    pub motion_blocking: Option<Vec<i64>>,
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
                    .map(|hm| expand_heightmap(hm.as_slice(), self.data_version))
                    .map(|hm| map.copy_from_slice(hm.as_slice()))
                    .is_some();

                if updated {
                    self.level.lazy_heightmap.replace(Some(map));
                    return;
                }
            }
            HeightMode::Calculate => {} // fall through to calc mode
        }

        for z in 0..16 {
            for x in 0..16 {
                // start at top until we hit a non-air block.
                for i in MIN_Y..MAX_Y {
                    let y = MAX_Y - i;
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
