use serde::Deserialize;
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Chunk<'a> {
    #[serde(rename = "DataVersion")]
    data_version: i32,

    #[serde(borrow)]
    level: Level<'a>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Level<'a> {
    #[serde(rename = "xPos")]
    x_pos: i32,

    #[serde(rename = "zPos")]
    z_pos: i32,

    biomes: &'a [u8],

    #[serde(borrow)]
    heightmaps: Heightmaps<'a>,

    // Ideally this would be done as a slice to avoid allocating the vector.
    // But there's no where to 'put' the slice of sections.
    #[serde(borrow)]
    sections: Vec<Section<'a>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Heightmaps<'a> {
    motion_blocking: &'a [u8],
    motion_blocking_no_leaves: &'a [u8],
    ocean_floor: &'a [u8],
    world_surface: &'a [u8],
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section<'a> {
    y: i8,
    block_states: Option<&'a [u8]>,
    palette: Option<Vec<Block<'a>>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Block<'a> {
    name: &'a str,
}
