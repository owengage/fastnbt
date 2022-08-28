use crate::{Block, BlockArchetype};

/// Initialize a `Block` from the given `block_id` and `data_value`.
pub fn init_default_block(block_id: u16, _data_value: u8) -> Block {
    assert!(
        block_id < 256,
        "init_default_block function only supports block ids in the 0..=255 range"
    );
    let block_name = block_name(block_id as u8);
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

/// Return the block name for a given pre-1.13 block id. The returned name does not contain the
/// `minecraft:` prefix.
///
/// Block ids can be up 12 bits, but here we only support 8-bit block ids, which are the ones used
/// in vanilla minecraft.
pub fn block_name(block_id: u8) -> &'static str {
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
