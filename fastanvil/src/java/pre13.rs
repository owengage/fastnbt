use std::convert::TryFrom;
use std::mem::MaybeUninit;
use std::ops::Range;
use std::sync::RwLock;

use fastnbt::ByteArray;
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::{biome::Biome, Block, BlockArchetype, Chunk, HeightMode};
use crate::{bits_per_block, expand_heightmap, Heightmaps, PackedBits, SectionLike, SectionTower};

/// This function creates an [OnceCell::new(); 256 * 256]
const fn uninit_block_list() -> [OnceCell<Block>; 256 * 256] {
    // We assume that since OnceCell::new is a const fn, the output will always be the same and
    // therefore can be safely copied. But OnceCell<NonCopy> does not implement Copy, do we need to
    // transmute it to a copyable type with the same size.
    let bit_pattern: OnceCell<Block> = OnceCell::new();
    // This is the result of converting OnceCell<Block> to bytes. We must use maybe uninitialized
    // bytes because of any possible padding in the source type, we are never allowed to read that
    // padding bytes.
    type OnceCellAsBytes = [MaybeUninit<u8>; std::mem::size_of::<OnceCell<Block>>()];
    // This is fine because we are transmuting to a type with the same size, and that type is a
    // [MaybeUninit<u8>; N] so even if the source type has padding this is not UB because we will
    // never read the padding bytes manually.
    let bit_pattern_bytes: OnceCellAsBytes = unsafe { std::mem::transmute(bit_pattern) };
    // This is fine because an uninitialized OnceCell can be copied, and a MaybeUninit type can
    // also be copied. Miri seems to accept that copying a MaybeUninit that may have padding is
    // perfectly fine as long as we never read that padding bytes manually.
    let a: [OnceCellAsBytes; 256 * 256] = [bit_pattern_bytes; 256 * 256];
    // This is fine because we obtained the OnceCellAsBytes type by transmuting a OnceCell<Block>
    // in the first place, so transmuting back will be safe.
    let a: [OnceCell<Block>; 256 * 256] = unsafe { std::mem::transmute(a) };

    a
}

// List of interned blocks, so we only create a Block with a specific id once, and we can return a
// reference to it in JavaChunk::block.
static BLOCK_LIST: [OnceCell<Block>; 256 * 256] = uninit_block_list();

/// Use this to manually register the conversion from numeric block id (1) to string block id
/// (minecraft:stone).
///
/// 12 bits of block_id, 4 bits of data_value.
///
/// Returns an error if the block with this id and data value has already been initialized, in that
/// case the old value is left intact.
pub fn init_block(block_id: u16, data_value: u8, block: Block) -> Result<(), Block> {
    assert!(block_id < (1u16 << 12));
    assert!(data_value < (1u8 << 4));
    let block_list_index = ((block_id as usize) << 4) + data_value as usize;

    BLOCK_LIST[block_list_index].set(block)
}

fn init_default_block(block_id: u16, _data_value: u8) -> Block {
    let block_name = block_name(block_id as u16);

    if block_name == "" {
        //panic!("Find block_name for id {}", block_id);
    }

    let block_name = format!("minecraft:{}", block_name);
    // TODO: add properties
    // This may be hard because the property name depends on the block id and on the
    // block_data, so that function will be very complex.
    let encoded = format!("{}|", block_name);

    Block {
        name: block_name,
        encoded,
        // TODO: use same logic as Block from BlockRaw
        archetype: BlockArchetype::Normal,
    }
}

// Block ids can be up 12 bits
const fn block_name(block_id: u16) -> &'static str {
    match block_id {
        0 => "air",
        1 => "stone",
        2 => "grass",
        3 => "dirt",
        4 => "cobblestone",
        5 => "planks",
        6 => "sapling",
        7 => "bedrock",
        8 => "flowing_water",
        9 => "water",
        10 => "flowing_lava",
        11 => "lava",
        12 => "sand",
        13 => "gravel",
        14 => "gold_ore",
        15 => "iron_ore",
        16 => "coal_ore",
        17 => "log",
        18 => "leaves",
        19 => "sponge",
        20 => "glass",
        21 => "lapis_ore",
        22 => "lapis_block",
        23 => "dispenser",
        24 => "sandstone",
        25 => "noteblock",
        26 => "bed",
        27 => "golden_rail",
        28 => "detector_rail",
        29 => "sticky_piston",
        30 => "web",
        31 => "tallgrass",
        32 => "deadbush",
        33 => "piston",
        34 => "piston_head",
        35 => "wool",
        37 => "yellow_flower",
        38 => "red_flower",
        39 => "brown_mushroom",
        40 => "red_mushroom",
        41 => "gold_block",
        42 => "iron_block",
        43 => "double_stone_slab",
        44 => "stone_slab",
        45 => "brick_block",
        46 => "tnt",
        47 => "bookshelf",
        48 => "mossy_cobblestone",
        49 => "obsidian",
        50 => "torch",
        51 => "fire",
        52 => "mob_spawner",
        53 => "oak_stairs",
        54 => "chest",
        55 => "redstone_wire",
        56 => "diamond_ore",
        57 => "diamond_block",
        58 => "crafting_table",
        59 => "wheat",
        60 => "farmland",
        61 => "furnace",
        62 => "lit_furnace",
        63 => "standing_sign",
        64 => "wooden_door",
        65 => "ladder",
        66 => "rail",
        67 => "stone_stairs",
        68 => "wall_sign",
        69 => "lever",
        70 => "stone_pressure_plate",
        71 => "iron_door",
        72 => "wooden_pressure_plate",
        73 => "redstone_ore",
        74 => "lit_redstone_ore",
        75 => "unlit_redstone_torch",
        76 => "redstone_torch",
        77 => "stone_button",
        78 => "snow_layer",
        79 => "ice",
        80 => "snow",
        81 => "cactus",
        82 => "clay",
        83 => "reeds",
        84 => "jukebox",
        85 => "fence",
        86 => "pumpkin",
        87 => "netherrack",
        88 => "soul_sand",
        89 => "glowstone",
        90 => "portal",
        91 => "lit_pumpkin",
        92 => "cake",
        93 => "unpowered_repeater",
        94 => "powered_repeater",
        95 => "stained_glass",
        96 => "trapdoor",
        97 => "monster_egg",
        98 => "stonebrick",
        99 => "brown_mushroom_block",
        100 => "red_mushroom_block",
        101 => "iron_bars",
        102 => "glass_pane",
        103 => "melon_block",
        // TODO: add more blocks
        _ => "",
    }
}

/// A Minecraft chunk.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JavaChunk {
    /// Only exists starting from 1.9
    pub data_version: Option<i32>,
    pub level: Level,
}

impl Chunk for JavaChunk {
    fn status(&self) -> String {
        // TODO: use LightPopulated and TerrainPopulated level flags to return a more accurate
        // status?
        "full".to_string()
    }

    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize {
        let mut heightmap = self.level.lazy_heightmap.read().unwrap();
        if heightmap.is_none() {
            drop(heightmap);
            self.recalculate_heightmap(mode);
            heightmap = self.level.lazy_heightmap.read().unwrap();
        }
        heightmap.unwrap()[z * 16 + x] as isize
    }

    fn biome(&self, x: usize, _y: isize, z: usize) -> Option<Biome> {
        let biomes = self.level.biomes.as_ref()?;

        // 1x1 columns stored z then x.
        let i = z * 16 + x;
        let biome = biomes[i];
        Biome::try_from(biome as i32).ok()
    }

    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block> {
        let sec = self.level.sections.as_ref()?.get_section_for_y(y)?;

        let sec_y = (y - sec.y as isize * 16) as usize;
        let idx: usize = (sec_y << 8) + (z << 4) + x;

        // Important: byte array can have negative values, we want to convert -1 into 255
        // so we cast first to u8, and then to usize
        let mut block_id = sec.blocks[idx] as u8 as usize;

        // Add extra bits from add field if present
        if let Some(add) = &sec.add {
            let mut add_id = add[idx / 2] as u8;
            if idx % 2 == 0 {
                // TODO: I am guessing the order here, 50% chance
                add_id = add_id & 0x0F;
            } else {
                add_id = (add_id & 0xF0) >> 4;
            }
            block_id += (add_id as usize) << 8;
        }

        let block_data = {
            let mut add_id = sec.data[idx / 2] as u8;
            if idx % 2 == 0 {
                add_id = add_id & 0x0F;
            } else {
                add_id = (add_id & 0xF0) >> 4;
            }

            add_id
        };
        let block_list_index = (block_id << 4) + block_data as usize;

        Some(
            BLOCK_LIST[block_list_index]
                .get_or_init(|| init_default_block(block_id as u16, block_data)),
        )
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

    pub biomes: Option<ByteArray>,

    /// Can be empty if the chunk hasn't been generated properly yet.
    pub sections: Option<SectionTower<Pre13Section>>,

    pub heightmaps: Option<Heightmaps>,

    #[serde(skip)]
    lazy_heightmap: RwLock<Option<[i16; 256]>>,
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
                        // TODO: does data_version matter before 1.9?
                        expand_heightmap(hm, y_min, self.data_version.unwrap_or(0))
                    })
                    .map(|hm| map.copy_from_slice(hm.as_slice()))
                    .is_some();

                if updated {
                    *self.level.lazy_heightmap.write().unwrap() = Some(map);
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

        *self.level.lazy_heightmap.write().unwrap() = Some(map);
    }
}

/// A vertical section of a chunk (ie a 16x16x16 block cube), for before 1.13.
///
/// Every possible block can be encoded using only 16 bits.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Pre13Section {
    pub y: i8,

    // 8 bits per block
    pub blocks: ByteArray,

    // 4 bits per block
    pub add: Option<ByteArray>,

    // 4 bits per block
    pub data: ByteArray,
}

impl SectionLike for Pre13Section {
    fn is_terminator(&self) -> bool {
        // TODO: does this break anything?
        false
    }

    fn y(&self) -> i8 {
        self.y
    }
}
