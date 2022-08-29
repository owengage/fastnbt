use std::convert::TryFrom;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Range;
use std::sync::RwLock;

use fastnbt::ByteArray;
use once_cell::sync::OnceCell;
use serde::Deserialize;

use crate::{biome::Biome, Block, Chunk, HeightMode};
use crate::{expand_heightmap, Heightmaps, SectionLike, SectionTower};

/// Conversion from numeric block ids to string based block names.
mod pre13_block_names;

/// This function creates an [OnceCell::new(); 256 * 16]
const fn uninit_block_list() -> [OnceCell<Block>; 256 * 16] {
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
    let a: [OnceCellAsBytes; 256 * 16] = [bit_pattern_bytes; 256 * 16];
    // This is fine because we obtained the OnceCellAsBytes type by transmuting a OnceCell<Block>
    // in the first place, so transmuting back will be safe.
    let a: [OnceCell<Block>; 256 * 16] = unsafe { std::mem::transmute(a) };

    a
}

// List of interned blocks, so we only create a Block with a specific id once, and we can return a
// reference to it in JavaChunk::block.
static BLOCK_LIST: [OnceCell<Block>; 256 * 16] = uninit_block_list();

/// Use this to manually register the conversion from numeric block id (1) to string block id
/// (minecraft:stone).
///
/// 8 bits of block_id, 4 bits of data_value.
///
/// If you need to support block ids greater than 255, use `set_custom_block_callback`.
///
/// Returns an error if the block with this id and data value has already been initialized, in that
/// case the old value is left intact.
pub fn init_block(block_id: u16, data_value: u8, block: Block) -> Result<(), Block> {
    assert!(block_id < (1u16 << 12));
    assert!(data_value < (1u8 << 4));
    let block_list_index = ((block_id as usize) << 4) + data_value as usize;

    BLOCK_LIST[block_list_index].set(block)
}

/// Function used to convert a block ids in the 256..=4095 range and a data value to static
/// references to `Block`.
pub type CustomBlockCallback = Box<dyn Send + Sync + Fn(u16, u8) -> Option<&'static Block>>;

/// User-controlled callback used to convert block ids in the 256..=4095 range and a data value to
/// static references to `Block`.
static CUSTOM_BLOCK_CALLBACK: OnceCell<RwLock<CustomBlockCallback>> = OnceCell::new();

/// Set a custom callback to convert block ids in the 256..=4095 range and a data value to static
/// references to `Block`. The callback can return `None` if the block id does not exist.
///
/// Returns the previously set callback.
pub fn set_custom_block_callback(f: CustomBlockCallback) -> CustomBlockCallback {
    std::mem::replace(
        &mut *CUSTOM_BLOCK_CALLBACK
            .get_or_init(|| RwLock::new(Box::new(|_block_id, _data_value| None)))
            .write()
            .unwrap(),
        f,
    )
}

fn custom_block_callback(block_id: u16, data_value: u8) -> Option<&'static Block> {
    (CUSTOM_BLOCK_CALLBACK
        .get_or_init(|| RwLock::new(Box::new(|_block_id, _data_value| None)))
        .read()
        .unwrap())(block_id, data_value)
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
        let raw_block = sec.block(x, sec_y, z);

        if (raw_block.0 as usize) < BLOCK_LIST.len() {
            Some(BLOCK_LIST[raw_block.0 as usize].get_or_init(|| {
                pre13_block_names::init_default_block(raw_block.block_id(), raw_block.data_value())
            }))
        } else {
            Some(
                custom_block_callback(raw_block.block_id(), raw_block.data_value())
                    .unwrap_or_else(|| panic!("Unknown raw block index {:?}. Use `set_custom_block_callback` to support this block id", raw_block)),
            )
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

impl Pre13Section {
    fn block(&self, x: usize, sec_y: usize, z: usize) -> RawBlock {
        let idx: usize = (sec_y << 8) + (z << 4) + x;

        // Important: byte array can have negative values, we want to convert -1 into 255
        // so we cast first to u8, and then to usize
        let mut block_id = self.blocks[idx] as u8 as usize;

        // Add extra bits from add field if present
        if let Some(add) = &self.add {
            let mut add_id = add[idx / 2] as u8;
            if idx % 2 == 0 {
                add_id &= 0x0F;
            } else {
                add_id = (add_id & 0xF0) >> 4;
            }
            block_id += (add_id as usize) << 8;
        }

        let block_data = {
            let mut add_id = self.data[idx / 2] as u8;
            if idx % 2 == 0 {
                add_id &= 0x0F;
            } else {
                add_id = (add_id & 0xF0) >> 4;
            }

            add_id
        };
        let block_list_index = (block_id << 4) + block_data as usize;

        RawBlock(block_list_index as u16)
    }
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

/// Raw block representation: block_id:data_value
///
/// block_id is 12 bits, data is 4 bits
#[derive(Default, PartialEq, Eq)]
pub struct RawBlock(u16);

impl fmt::Debug for RawBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawBlock({}:{})", self.block_id(), self.data_value())
    }
}

impl RawBlock {
    /// Return 12-bit block id
    pub fn block_id(&self) -> u16 {
        self.0 >> 4
    }

    /// Return 4-bit data value (block variant)
    pub fn data_value(&self) -> u8 {
        (self.0 & 0x0F) as u8
    }
}
