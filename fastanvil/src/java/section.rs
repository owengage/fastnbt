use serde::Deserialize;

use crate::{BiomeData, Block, BlockData, Pre18Blockstates};

pub trait SectionLike {
    fn is_terminator(&self) -> bool;
    fn y(&self) -> i8;
}

/// A vertical section of a chunk (ie a 16x16x16 block cube)
#[derive(Deserialize, Debug)]
pub struct Section {
    #[serde(rename = "Y")]
    pub y: i8,

    // TODO: Default instead
    pub block_states: Option<BlockData<Block>>,

    #[serde(default)]
    pub biomes: BiomeData<String>, // TODO: Biome type?
}

impl SectionLike for Section {
    fn is_terminator(&self) -> bool {
        false
    }

    fn y(&self) -> i8 {
        self.y
    }
}

/// A vertical section of a chunk (ie a 16x16x16 block cube), for before 1.18.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Pre18Section {
    pub y: i8,

    pub block_states: Option<Pre18Blockstates>,

    #[serde(default)]
    pub palette: Vec<Block>,
}

impl SectionLike for Pre18Section {
    fn is_terminator(&self) -> bool {
        self.palette.is_empty() && self.block_states.is_none()
    }

    fn y(&self) -> i8 {
        self.y
    }
}
