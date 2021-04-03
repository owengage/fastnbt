use std::{cell::RefCell, collections::HashMap, convert::TryFrom};

use lazy_static::lazy_static;
use serde::Deserialize;

use crate::{bits_per_block, expand_heightmap, Chunk, HeightMode, PackedBits, MAX_Y, MIN_Y};

use super::biome::Biome;

lazy_static! {
    static ref AIR: Block = Block {
        name: "minecraft:air".to_owned(),
        properties: HashMap::new(),
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
        let biomes = self.level.biomes.as_ref().unwrap();

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
                let after1_17 = self.data_version >= 2695;
                let y_shifted = if after1_17 { y + 64 } else { y } as usize;

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
        let sec = self.get_section_for_y(y)?;

        // If a section is entirely air, then the block states are missing
        // entirely, presumably to save space.
        if sec.block_states.is_none() {
            return Some(&AIR);
        }

        let sec_y = y - sec.y as isize * 16;
        let state_index = (sec_y as usize * 16 * 16) + z * 16 + x;

        if *sec.unpacked_states.borrow() == None {
            let bits_per_item = bits_per_block(sec.palette.len());
            *sec.unpacked_states.borrow_mut() = Some([0; 16 * 16 * 16]);

            let mut states = sec.unpacked_states.borrow_mut();
            let buf = states.as_mut().unwrap();

            sec.block_states
                .as_ref()?
                .unpack_blockstates(bits_per_item, &mut buf[..]);
        }

        let pal_index = sec.unpacked_states.borrow().as_ref().unwrap()[state_index] as usize;

        sec.palette.get(pal_index)
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

    #[serde(default)]
    pub biomes: Option<Vec<i32>>,

    /// Can be empty if the chunk hasn't been generated properly yet.
    pub sections: Option<Vec<Section>>,

    pub heightmaps: Option<Heightmaps>,

    // Status of the chunk. Typically anything except 'full' means the chunk
    // hasn't been fully generated yet. We use this to skip chunks on map edges
    // that haven't been fully generated yet.
    pub status: String,

    // Maps the y value from each section to the index in the `sections` field.
    // Makes it quicker to find the correct section when all you have is the height.
    #[serde(skip)]
    #[serde(default)]
    sec_map: RefCell<HashMap<i8, usize>>,

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

    pub block_states: Option<PackedBits>,

    #[serde(default)]
    pub palette: Vec<Block>,

    #[serde(skip)]
    unpacked_states: RefCell<Option<[u16; 16 * 16 * 16]>>,
}

/// A block within the world.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Block {
    pub name: String,

    #[serde(default)]
    pub properties: HashMap<String, String>,
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
                        .contains(&block.unwrap().name.as_str())
                    {
                        map[z * 16 + x] = y as i16;
                        break;
                    }
                }
            }
        }

        self.level.lazy_heightmap.replace(Some(map));
    }

    fn calculate_sec_map(&self) {
        let mut map = self.level.sec_map.borrow_mut();

        for (i, sec) in self.level.sections.iter().flatten().enumerate() {
            map.insert(sec.y, i);
        }
    }

    fn get_section_for_y(&self, y: isize) -> Option<&Section> {
        if self.level.sections.as_ref()?.is_empty() {
            return None;
        }

        if self.level.sec_map.borrow().is_empty() {
            self.calculate_sec_map();
        }

        // Need to be careful. y = -5 should return -1. If we did normal integer
        // division 5/16 would give us 0.
        let containing_section_y = ((y as f64) / 16.0).floor() as i8;

        let section_index = *self.level.sec_map.borrow().get(&(containing_section_y))?;

        let sec = self.level.sections.as_ref()?.get(section_index);
        sec
    }
}

impl Block {
    /// Creates a string of the format "id|prop1=val1,prop2=val2". The
    /// properties are ordered lexigraphically. This somewhat matches the way
    /// Minecraft stores variants in blockstates, but with the block ID/name
    /// prepended.
    pub fn encoded_description(&self) -> String {
        let mut id = self.name.to_string() + "|";
        let mut sep = "";

        let mut props = self
            .properties
            .iter()
            .filter(|(k, _)| **k != "waterlogged") // TODO: Handle water logging. See note below
            .filter(|(k, _)| **k != "powered") // TODO: Handle power
            .collect::<Vec<_>>();

        // need to sort the properties for a consistent ID
        props.sort();

        for (k, v) in props {
            id = id + sep + k + "=" + v;
            sep = ",";
        }

        id

        // Note: If we want to handle water logging, we're going to have to
        // remove the filter here and handle it in whatever parses the encoded
        // ID itself. This will probably be pretty ugly. It can probably be
        // contained in the palette generation code entirely, so shouldn't
        // impact speed too hard.
    }
}
