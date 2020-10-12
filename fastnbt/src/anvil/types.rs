use std::{collections::HashMap, convert::TryFrom};

use crate::anvil::bits::PackedBits;
use byteorder::{BigEndian, ReadBytesExt};
use serde::Deserialize;

use super::biome::Biome;

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Chunk<'a> {
    #[serde(rename = "DataVersion")]
    pub data_version: i32,

    #[serde(borrow)]
    pub level: Level<'a>,
}

impl<'a> Chunk<'a> {
    pub fn id_of(&mut self, x: usize, y: usize, z: usize) -> Option<String> {
        if self.level.sections.is_empty() {
            return None;
        }

        // First section is sometimes y = -1.
        // If that's the case we want to add one to the section we attempt to get.
        if self.level.sections[0].y != 0 {}
        let containing_section_y = (y / 16) as isize - (self.level.sections[0].y as isize);

        let sec = self.level.sections.get_mut(containing_section_y as usize);

        if let Some(sec) = sec {
            if (sec.y as usize) * 16 > y {}
            let sec_y = y - sec.y as usize * 16;
            let state_index = (sec_y as usize * 16 * 16) + x * 16 + z;

            if sec.unpacked_states == None {
                let bits_per_item = super::bits::bits_per_block(sec.palette.as_ref()?.len());
                let mut buf = vec![0; 16 * 16 * 16];

                sec.block_states
                    .as_ref()?
                    .unpack_into(bits_per_item, &mut buf[..]);

                sec.unpacked_states = Some(buf);
            }

            let pal_index = sec.unpacked_states.as_ref()?[state_index] as usize;
            Some(sec.palette.as_ref()?[pal_index].encoded_id())
        } else {
            None
        }
    }

    pub fn height_of(&mut self, x: usize, z: usize) -> Option<usize> {
        let ref mut maps = self.level.heightmaps;

        if maps.unpacked_motion_blocking == None {
            let mut buf = vec![0u16; 16 * 16];
            maps.motion_blocking.as_ref()?.unpack_into(9, &mut buf[..]);
            maps.unpacked_motion_blocking = Some(buf);
        }

        Some(maps.unpacked_motion_blocking.as_ref()?[x * 16 + z] as usize)
    }

    pub fn biome_of(&self, x: usize, _y: usize, z: usize) -> Option<Biome> {
        // TODO: Take into account height. For overworld this doesn't matter (at least not yet)
        // TODO: Make use of data version?

        // For biome len of 1024,
        //  it's 4x4x4 sets of blocks stored by z then x then y (+1 moves one in z)
        //  for overworld theres no vertical chunks so it looks like only first 16 values are used.
        // For biome len of 256, it's chunk 1x1 columns stored z then x.

        let biomes = self.level.biomes?;

        if biomes.len() == 1024 * 4 {
            // Minecraft 1.16
            let i = 4 * ((x / 4) * 4 + (z / 4));
            let biome = (&biomes[i..]).read_i32::<BigEndian>().ok()?;

            Biome::try_from(biome).ok()
        } else if biomes.len() == 256 * 4 {
            // Minecraft 1.15 (and past?)
            let i = 4 * (x * 16 + z);
            let biome = (&biomes[i..]).read_i32::<BigEndian>().ok()?;
            Biome::try_from(biome).ok()
        } else {
            None
        }
    }
}

/// A level describes the contents of the chunk in the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Level<'a> {
    #[serde(rename = "xPos")]
    pub x_pos: i32,

    #[serde(rename = "zPos")]
    pub z_pos: i32,

    pub biomes: Option<&'a [u8]>,

    #[serde(borrow)]
    pub heightmaps: Heightmaps<'a>,

    // Ideally this would be done as a slice to avoid allocating the vector.
    // But there's no where to 'put' the slice of sections.
    pub sections: Vec<Section<'a>>,
}

/// Various heightmaps kept up to date by Minecraft.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Heightmaps<'a> {
    #[serde(borrow)]
    pub motion_blocking: Option<PackedBits<'a>>,
    pub motion_blocking_no_leaves: Option<PackedBits<'a>>,
    pub ocean_floor: Option<PackedBits<'a>>,
    pub world_surface: Option<PackedBits<'a>>,

    #[serde(skip)]
    unpacked_motion_blocking: Option<Vec<u16>>,
}

/// A vertical section of a chunk (ie a 16x16x16 block cube)
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section<'a> {
    pub y: i8,

    #[serde(borrow)]
    pub block_states: Option<PackedBits<'a>>,
    pub palette: Option<Vec<Block<'a>>>,

    #[serde(skip)]
    unpacked_states: Option<Vec<u16>>,
}

/// A block within the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Block<'a> {
    pub name: &'a str,
    pub properties: Option<HashMap<&'a str, &'a str>>,
}

impl<'a> Block<'a> {
    fn encoded_id(&self) -> String {
        // todo sort
        match self.properties {
            None => self.name.to_string() + "|",
            Some(ref props) => {
                let mut id = self.name.to_string();
                let mut sep = "|";
                for (k, v) in props {
                    id = id + sep + k + ":" + v;
                    sep = ",";
                }

                id
            }
        }
    }
}
