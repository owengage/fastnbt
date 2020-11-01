use std::{
    collections::HashMap,
    convert::TryFrom,
    io::{Read, Seek},
};

use byteorder::{BigEndian, ReadBytesExt};
use serde::Deserialize;

use crate::{bits_per_block, PackedBits};

use super::{
    biome::{self, Biome},
    Region,
};

pub type Rgba = [u8; 4];

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Chunk {
    pub data_version: i32,
    pub level: Level,
}

/// A level describes the contents of the chunk in the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Level {
    #[serde(rename = "xPos")]
    pub x_pos: i32,

    #[serde(rename = "zPos")]
    pub z_pos: i32,

    pub biomes: Option<Vec<u8>>,

    pub heightmaps: Option<Heightmaps>,

    // Old chunk formats can store a plain heightmap in an IntArray here.
    #[serde(rename = "HeightMap")]
    pub old_heightmap: Option<Vec<i32>>,

    /// Ideally this would be done as a slice to avoid allocating the vector.
    /// But there's no where to 'put' the slice of sections.
    ///
    /// Can be empty if the chunk hasn't been generated properly yet.
    pub sections: Option<Vec<Section>>,

    // Status of the chunk. Typically anything except 'full' means the chunk
    // hasn't been fully generated yet. We use this to skip chunks on map edges
    // that haven't been fully generated yet.
    pub status: String,

    // Maps the y value from each section to the index in the `sections` field.
    // Makes it quicker to find the correct section when all you have is the height.
    #[serde(skip)]
    sec_map: Option<HashMap<i8, usize>>,
}

/// Various heightmaps kept up to date by Minecraft.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Heightmaps {
    pub motion_blocking: Option<Vec<i64>>,
    pub motion_blocking_no_leaves: Option<Vec<i64>>,
    pub ocean_floor: Option<Vec<i64>>,
    pub world_surface: Option<Vec<i64>>,

    #[serde(skip)]
    unpacked_motion_blocking: Option<Vec<u16>>,
}

/// A vertical section of a chunk (ie a 16x16x16 block cube)
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub y: i8,

    pub block_states: Option<Vec<i64>>,
    pub palette: Option<Vec<Block>>,

    // Perhaps a little large to potentially end up on the stack? 8 KiB.
    #[serde(skip)]
    unpacked_states: Option<Vec<u16>>,
}

/// A block within the world.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Block {
    pub name: String,

    #[serde(default)]
    pub properties: HashMap<String, String>,
}

impl Chunk {
    pub fn block(&mut self, x: usize, y: usize, z: usize) -> Option<&Block> {
        let sec = self.get_section_for_y(y)?;

        if (sec.y as usize) * 16 > y {}
        let sec_y = y - sec.y as usize * 16;
        let state_index = (sec_y as usize * 16 * 16) + x * 16 + z;

        if sec.unpacked_states == None {
            sec.unpacked_states = Some(crate::expand_blockstates(
                sec.block_states.as_ref()?.as_slice(),
                bits_per_block(sec.palette.as_ref()?.len()),
            ));
        }

        let pal_index = sec.unpacked_states.as_ref()?[state_index] as usize;
        sec.palette.as_ref()?.get(pal_index)
    }

    pub fn height_of(&mut self, x: usize, z: usize) -> Option<usize> {
        let ref mut maps = self.level.heightmaps;

        match maps {
            Some(maps) => {
                if maps.unpacked_motion_blocking == None {
                    maps.unpacked_motion_blocking = Some(crate::expand_heightmap(
                        maps.motion_blocking.as_ref()?.as_slice(),
                    ));
                }

                Some(maps.unpacked_motion_blocking.as_ref()?[x * 16 + z] as usize)
            }
            None => self // Older style heightmap found. Much simpler, just an int per column.
                .level
                .old_heightmap
                .as_ref()
                .map(|v| v[x * 16 + z] as usize),
        }
    }

    pub fn biome_of(&self, x: usize, _y: usize, z: usize) -> Option<Biome> {
        // TODO: Take into account height. For overworld this doesn't matter (at least not yet)
        // TODO: Make use of data version?

        // For biome len of 1024,
        //  it's 4x4x4 sets of blocks stored by z then x then y (+1 moves one in z)
        //  for overworld theres no vertical chunks so it looks like only first 16 values are used.
        // For biome len of 256, it's chunk 1x1 columns stored z then x.

        let biomes = self.level.biomes.as_ref()?;

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

    fn calculate_sec_map(&mut self) {
        self.level.sec_map = Some(HashMap::new());
        let map = self.level.sec_map.as_mut().unwrap();

        for (i, sec) in self.level.sections.iter().flatten().enumerate() {
            map.insert(sec.y, i);
        }
    }

    fn get_section_for_y(&mut self, y: usize) -> Option<&mut Section> {
        if self.level.sections.as_ref()?.is_empty() {
            return None;
        }

        if self.level.sec_map.is_none() {
            self.calculate_sec_map();
        }

        let containing_section_y = y / 16;
        let section_index = self
            .level
            .sec_map
            .as_ref()
            .unwrap() // calculate_sec_map() make sure this is valid.
            .get(&(containing_section_y as i8))?;

        let sec = self.level.sections.as_mut()?.get_mut(*section_index);
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

        // need to sort the properties for a consistent ID
        let mut props = self
            .properties
            .iter()
            .filter(|(k, _)| **k != "waterlogged") // TODO: Handle water logging. See note below
            .collect::<Vec<_>>();
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
        // impact speed to hard.
    }
}

/// ChunkRender objects can render a given chunk. What they render to is
/// entirely up to the implementation.
pub trait ChunkRender {
    /// Draw the given chunk.
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk);

    /// Draw the invalid chunk. This means that the chunk was not of an expected
    /// form and couldn't be deserialized into a chunk object.
    fn draw_invalid(&mut self, xc_rel: usize, zc_rel: usize);
}

/// Palette can be used to take a block description to produce a colour that it
/// should render to.
pub trait Palette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba;
}

pub trait IntoMap {
    fn into_map(self) -> RegionMap<Rgba>;
}

pub struct RegionMap<T> {
    pub data: Vec<T>,
    pub x_region: isize,
    pub z_region: isize,
}

impl<T: Clone> RegionMap<T> {
    pub fn new(x_region: isize, z_region: isize, default: T) -> Self {
        let mut data: Vec<T> = Vec::new();
        data.resize(16 * 16 * 32 * 32, default);
        Self {
            data,
            x_region,
            z_region,
        }
    }

    pub fn chunk_mut(&mut self, x: usize, z: usize) -> &mut [T] {
        let len = 16 * 16;
        let begin = (x * 32 + z) * len;
        &mut self.data[begin..begin + len]
    }

    pub fn chunk(&self, x: usize, z: usize) -> &[T] {
        let len = 16 * 16;
        let begin = (x * 32 + z) * len;
        &self.data[begin..begin + len]
    }
}

#[derive(Debug)]
pub enum DrawError {
    ParseAnvil(super::Error),
    ParseNbt(fastnbt::error::Error),
    IO(std::io::Error),
    MissingHeightMap,
    InvalidPalette,
}

impl From<super::Error> for DrawError {
    fn from(err: super::Error) -> DrawError {
        DrawError::ParseAnvil(err)
    }
}

impl From<fastnbt::error::Error> for DrawError {
    fn from(err: fastnbt::error::Error) -> DrawError {
        DrawError::ParseNbt(err)
    }
}

impl From<std::io::Error> for DrawError {
    fn from(err: std::io::Error) -> Self {
        DrawError::IO(err)
    }
}

pub type DrawResult<T> = std::result::Result<T, DrawError>;

pub fn parse_region<F: ChunkRender + ?Sized, RS>(
    mut region: Region<RS>,
    draw_to: &mut F,
) -> DrawResult<()>
where
    RS: Read + Seek,
{
    let closure = |x: usize, z: usize, buf: &Vec<u8>| {
        let chunk = nbt::de::from_reader(buf.as_slice());
        match chunk {
            Ok(mut chunk) => draw_to.draw(x, z, &mut chunk),
            Err(e) => {
                draw_to.draw_invalid(x, z);
                println!("{:?}", e);
            }
        }
    };

    region.for_each_chunk(closure)?;
    Ok(())
}

pub struct RenderedPalette {
    pub blockstates: std::collections::HashMap<String, Rgba>,
    pub grass: image::RgbaImage,
    pub foliage: image::RgbaImage,
}

impl RenderedPalette {
    fn pick_grass(&self, b: Option<Biome>) -> Rgba {
        b.map(|b| {
            let climate = biome::climate(b);
            let t = climate.temperature.min(1.).max(0.);
            let r = climate.rainfall.min(1.).max(0.) * t;

            let t = 255 - (t * 255.).ceil() as u32;
            let r = 255 - (r * 255.).ceil() as u32;

            self.grass.get_pixel(t, r).0
        })
        .unwrap_or([0, 0, 0, 0])
    }

    fn pick_foliage(&self, b: Option<Biome>) -> Rgba {
        b.map(|b| {
            let climate = biome::climate(b);
            let t = climate.temperature.min(1.).max(0.);
            let r = climate.rainfall.min(1.).max(0.) * t;

            let t = 255 - (t * 255.).ceil() as u32;
            let r = 255 - (r * 255.).ceil() as u32;

            self.foliage.get_pixel(t, r).0
        })
        .unwrap_or([0, 0, 0, 0])
    }

    fn pick_water(&self, b: Option<Biome>) -> Rgba {
        b.map(|b| match b {
            Biome::Swamp => [0x61, 0x7B, 0x64, 255],
            Biome::River => [0x3F, 0x76, 0xE4, 255],
            Biome::Ocean => [0x3F, 0x76, 0xE4, 255],
            Biome::LukewarmOcean => [0x45, 0xAD, 0xF2, 255],
            Biome::WarmOcean => [0x43, 0xD5, 0xEE, 255],
            Biome::ColdOcean => [0x3D, 0x57, 0xD6, 255],
            Biome::FrozenRiver => [0x39, 0x38, 0xC9, 255],
            Biome::FrozenOcean => [0x39, 0x38, 0xC9, 255],
            _ => [0x3f, 0x76, 0xe4, 255],
        })
        .unwrap_or([0x3f, 0x76, 0xe4, 255])
    }
}

impl Palette for RenderedPalette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba {
        let missing_colour = [255, 0, 255, 255];
        let snow_block: Block = Block {
            name: "minecraft:snow_block".to_string(),
            properties: HashMap::new(),
        };

        // A bunch of blocks in the game seem to be special cased outside of the
        // blockstate/model mechanism. For example leaves get coloured based on
        // the tree type and the biome type, but this is not encoded in the
        // blockstate or the model.
        //
        // This means we have to do a bunch of complex conditional logic in one
        // of the most called functions. Yuck.
        match block.name.strip_prefix("minecraft:") {
            Some(id) => {
                match id {
                    "grass_block" => {
                        let snowy = block
                            .properties
                            .get("snowy")
                            .map(|s| *s == "true")
                            .unwrap_or(false);

                        if snowy {
                            return self.pick(&snow_block, biome);
                        } else {
                            return self.pick_grass(biome);
                        };
                    }
                    "water" => return self.pick_water(biome),
                    "oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves" => {
                        return self.pick_foliage(biome)
                    }
                    "birch_leaves" => {
                        return [0x80, 0xa7, 0x55, 255]; // game hardcodes this
                    }
                    "spruce_leaves" => {
                        return [0x61, 0x99, 0x61, 255]; // game hardcodes this
                    }
                    // Kelp and seagrass don't look like much from the top as
                    // they're flat. Maybe in future hard code a green tint to make
                    // it show up?
                    "kelp" | "seagrass" | "tall_seagrass" => {
                        return self.pick_water(biome);
                    }
                    "snow" => {
                        return self.pick(&snow_block, biome);
                    }
                    // Occurs a lot for the end, as layer 0 will be air in the void.
                    // Rendering it black makes sense in the end, but might look
                    // weird if it ends up elsewhere.
                    "air" => {
                        return [0, 0, 0, 0];
                    }
                    "cave_air" => {
                        return [255, 0, 0, 255]; // when does this happen??
                    }
                    // Otherwise fall through to the general mechanism.
                    _ => {}
                }
            }
            None => {}
        }

        let col = self.blockstates.get(&block.encoded_description());
        match col {
            Some(c) => *c,
            None => {
                //println!("could not draw {}", block.name);
                missing_colour
            }
        }
    }
}

pub struct RegionBlockDrawer<'a, P: Palette + ?Sized> {
    pub map: RegionMap<Rgba>,
    pub palette: &'a P,
    pub processed_chunks: usize,
    pub painted_pixels: usize,
}

impl<'a, P: Palette + ?Sized> RegionBlockDrawer<'a, P> {
    pub fn new(map: RegionMap<Rgba>, palette: &'a P) -> Self {
        Self {
            map,
            palette,
            processed_chunks: 0,
            painted_pixels: 0,
        }
    }
}

impl<'a, P: Palette + ?Sized> IntoMap for RegionBlockDrawer<'a, P> {
    fn into_map(self) -> RegionMap<Rgba> {
        self.map
    }
}

impl<'a, P: Palette + ?Sized> ChunkRender for RegionBlockDrawer<'a, P> {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk) {
        let data = self.map.chunk_mut(xc_rel, zc_rel);
        self.processed_chunks += 1;

        if chunk.level.status != "full" && chunk.level.status != "spawn" {
            // Chunks that have been fully generated will have a 'full' status.
            // Skip chunks that don't; the way they render is unpredictable.
            return;
        }

        // if !(zc_rel == 18 && (xc_rel == 22 || xc_rel == 23)) {
        //     return;
        // }

        //println!("{:#?}", chunk);
        let mut draw_cross = false;

        for z in 0..16 {
            for x in 0..16 {
                let height = match chunk.height_of(x, z) {
                    Some(height) => height,
                    None => {
                        let pixel = &mut data[x * 16 + z];
                        *pixel = [255, 0, 0, 255];
                        draw_cross = true;
                        continue;
                    }
                };

                let height = if height == 0 { 0 } else { height - 1 }; // -1 because we want the block below the air.
                let biome = chunk.biome_of(x, height, z);
                let block = chunk.block(x, height, z);

                let colour = match block {
                    Some(ref block) => self.palette.pick(&block, biome),
                    None => [0, 0, 0, 0], // if no ID is given the block doesn't actually exist in the world.
                };

                //println!("{:?}: {:?}, height {}", material, colour, height);

                let pixel = &mut data[x * 16 + z];
                *pixel = colour;
                self.painted_pixels += 1;
            }
        }

        if draw_cross {
            self.draw_invalid(xc_rel, zc_rel);
        }
    }

    fn draw_invalid(&mut self, xc_rel: usize, zc_rel: usize) {
        let data = self.map.chunk_mut(xc_rel, zc_rel);

        // Draw a red cross over the chunk
        for z in 0..16isize {
            for x in 0..16isize {
                let pixel = &mut data[x as usize * 16 + z as usize];

                *pixel = if (x - z).abs() < 3 || (x - (16 - z)).abs() < 3 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 0, 0]
                }
            }
        }
    }
}
