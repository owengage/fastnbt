use serde::Deserialize;

use crate::{biome::Biome, BiomeData, Block, BlockData};

pub trait SectionLike {
    fn is_terminator(&self) -> bool;
    fn y(&self) -> i8;
}

/// A vertical section of a chunk (ie a 16x16x16 block cube)
#[derive(Deserialize, Debug)]
pub struct Section {
    #[serde(rename = "Y")]
    pub y: i8,

    #[serde(default)]
    pub block_states: BlockData<Block>,

    #[serde(default)]
    pub biomes: BiomeData<Biome>,
}

impl SectionLike for Section {
    fn is_terminator(&self) -> bool {
        false
    }

    fn y(&self) -> i8 {
        self.y
    }
}
