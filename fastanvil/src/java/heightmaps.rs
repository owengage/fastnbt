use fastnbt::LongArray;
use serde::Deserialize;

/// Various heightmaps kept up to date by Minecraft.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Heightmaps {
    pub motion_blocking: Option<LongArray>,
    //pub motion_blocking_no_leaves: Option<Heightmap>,
    //pub ocean_floor: Option<Heightmap>,
    //pub world_surface: Option<Heightmap>,
}
