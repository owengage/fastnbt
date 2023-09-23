use crate::{Block, BlockArchetype};

macro_rules! coloured_block {
    ($a:expr, $b:expr) => {
        {
            let col = match $b & 0b1111 {
                0 => "white",
                1 => "orange",
                2 => "magenta",
                3 => "light_blue",
                4 => "yellow",
                5 => "lime",
                6 => "pink",
                7 => "gray",
                8 => "light_gray",
                9 => "cyan",
                10 => "purple",
                11 => "blue",
                12 => "brown",
                13 => "green",
                14 => "red",
                15 => "black",
                _ => unreachable!(),
            };
            Block {
                name: format!("minecraft:{col}_{}", $a),
                encoded: format!("minecraft:{col}_{}|", $a),
                archetype: BlockArchetype::Normal,
            }
        }
    }
}

/// Initialize a `Block` from the given `block_id` and `data_value`.
pub fn init_default_block(block_id: u16, data_value: u8) -> Block {
    assert!(
        block_id < 256,
        "init_default_block function only supports block ids in the 0..=255 range"
    );
    let block_name = block_name(block_id as u8);

    modern_block(block_name, data_value)
}

fn modern_block(block_name: &'static str, data_value: u8) -> Block {
    let encoded = format!("{}|", block_name);
    let ns = |s| format!("minecraft:{s}"); // add namespace
    let enc0 = |s| format!("minecraft:{s}|");

    // This function will get very large and complicated, need some way to break
    // it down. Could definitely use some macros for things like the wood/leaf types.

    match block_name {
        "double_wooden_slab" => {
            let kind = data_value & 0b0111;
            let kind = match kind {
                0 => "oak",
                1 => "spruce",
                2 => "birch",
                3 => "jungle",
                4 => "acacia",
                5 => "dark_oak",
                6 | 7 => "invalid_double_wooden_slab",
                _ => unreachable!()
            };
            Block {
                name: format!("{kind}_slab"),
                encoded: format!("{kind}_slab|type=double"),
                archetype: BlockArchetype::Normal,
            }
        }
        "wooden_slab" => {
            let kind = data_value & 0b0111;
            let kind = match kind {
                0 => "oak",
                1 => "spruce",
                2 => "birch",
                3 => "jungle",
                4 => "acacia",
                5 => "dark_oak",
                6 | 7 => "invalid_wooden_slab",
                _ => unreachable!()
            };
            let top = data_value & 0b1000;
            let top = match top {
                0 => "bottom",
                1 => "top",
                _ => unreachable!(),
            };
            Block {
                name: format!("{kind}_slab"),
                encoded: format!("{kind}_slab|type={top}"),
                archetype: BlockArchetype::Normal,
            }
        }
        "flower" => {
            let kind = data_value & 0b1111;
            let kind = match kind {
                0 => "poppy",
                1 => "blue_orchid",
                2 => "allium",
                3 => "azure_bluet",
                4 => "red_tulip",
                5 => "orange_tulip",
                6 => "white_tulip",
                7 => "pink_tulip",
                8 => "oxeye_daisy",
                9 | 10 | 11 | 12 | 13 | 14 | 15 => "invalid_flower",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "red_sandstone" => {
            let kind = data_value & 0b0011;
            let kind = match kind {
                0 => "red_sandstone",
                1 => "chiseled_red_sandstone",
                2 => "smooth_red_sandstone",
                3 => "invalid_red_sandstone",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "sandstone" => {
            let kind = data_value & 0b0011;
            let kind = match kind {
                0 => "sandstone",
                1 => "chiseled_sandstone",
                2 => "smooth_sandstone",
                3 => "invalid_sandstone",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "double_stone_slab" => {
            let kind = data_value & 0b0111;
            let kind = match kind {
                0 => "stone",
                1 => "sandstone",
                2 => "stone_wooden",
                3 => "cobblestone",
                4 => "bricks",
                5 => "stone_brick",
                6 => "nether_brick",
                7 => "quartz",
                _ => unreachable!(),
            };
            Block {
                name: format!("{kind}_slab"),
                encoded: format!("{kind}_slab|type=double"),
                archetype: BlockArchetype::Normal,
            }
        }
        "stone_slab" => {
            let kind = data_value & 0b0111;
            let kind = match kind {
                0 => "stone",
                1 => "sandstone",
                2 => "stone_wooden",
                3 => "cobblestone",
                4 => "bricks",
                5 => "stone_brick",
                6 => "nether_brick",
                7 => "quartz",
                _ => unreachable!(),
            };
            let top = data_value & 0b1000;
            let top = match top {
                0 => "bottom",
                1 => "top",
                _ => unreachable!(),
            };
            Block {
                name: format!("{kind}_slab"),
                encoded: format!("{kind}_slab|type={top}"),
                archetype: BlockArchetype::Normal,
            }
        }
        "double_stone_slab2" => {
            Block {
                name: ns("red_sandstone_slab"),
                encoded: enc0("red_sandstone_slab|type=double"),
                archetype: BlockArchetype::Normal,
            }
        }
        "stone_slab2" => {
            let top = data_value & 0b1000;
            let top = match top {
                0 => "bottom",
                8 => "top",
                _ => unreachable!(),
            };
            Block {
                name: ns("red_sandstone_slab"),
                encoded: format!("red_sandstone_slab|type={top}"),
                archetype: BlockArchetype::Normal,
            }
        }
        "stained_glass" => {
            coloured_block!("stained_glass", data_value)
        }
        "wool" => {
            coloured_block!("wool", data_value)
        }
        "carpet" => {
            coloured_block!("carpet", data_value)
        }
        "sand" => {
            let kind = data_value & 0b0001;
            let kind = match kind {
                0 => "sand",
                1 => "red_sand",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "sapling" => {
            let kind = data_value & 0b0111;
            let kind = match kind {
                0 => "oak_sapling",
                1 => "spruce_sapling",
                2 => "birch_sapling",
                3 => "jungle_sapling",
                4 => "acacia_sapling",
                5 => "dark_oak_sapling",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "dirt" => {
            let kind = data_value & 0b0011;
            let kind = match kind {
                0 => "dirt", 
                1 => "coarse_dirt",
                2 => "podzol",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "stone" => {
            let kind = data_value & 0b0111;
            let kind = match kind {
                0 => "stone",
                1 => "granite",
                2 => "polished_granite",
                3 => "diorite",
                4 => "polished_diorite",
                5 => "andesite",
                6 => "polished_andesite",
                _ => unreachable!(),
            };
            Block {
                name: ns(kind),
                encoded: enc0(kind),
                archetype: BlockArchetype::Normal,
            }
        }
        "leaves" => {
            let leaf = data_value & 0b0011;
            let leaf = match leaf {
                0 => "oak_leaves",
                1 => "spruce_leaves",
                2 => "birch_leaves",
                3 => "jungle_leaves",
                _ => unreachable!(),
            };
            Block {
                name: ns(leaf),
                encoded: enc0(leaf),
                archetype: BlockArchetype::Normal,
            }
        }
        "leaves2" => {
            let leaf = data_value & 0b0011;
            let leaf = match leaf {
                0 => "acacia_leaves",
                1 => "dark_oak_leaves",
                2 | 3 => "invalid_leaves",
                _ => unreachable!(),
            };
            Block {
                name: ns(leaf),
                encoded: enc0(leaf),
                archetype: BlockArchetype::Normal,
            }
        }
        "log" => {
            let log = data_value & 0b0011;
            let axis = data_value & 0b1100;
            let axis = match axis {
                0 => "y",
                1 => "x",
                2 => "z",
                3 => "z", // this actually represents all bark.
                _ => unreachable!(),
            };
            let log = match log {
                0 => "oak_log",
                1 => "spruce_log",
                2 => "birch_log",
                3 => "jungle_log",
                _ => unreachable!(),
            };
            Block {
                name: ns(log),
                encoded: format!("minecraft:{log}|axis={axis}"),
                archetype: BlockArchetype::Normal,
            }
        }
        "log2" => {
            let log = data_value & 0b0011;
            let axis = data_value & 0b1100;
            let axis = match axis {
                0 => "y",
                1 => "x",
                2 => "z",
                3 => "z", // this actually represents all bark.
                _ => unreachable!(),
            };
            let log = match log {
                0 => "acacia_log",
                1 => "dark_oak_log",
                2 | 3 => "invalid_log",
                _ => unreachable!(),
            };
            Block {
                name: ns(log),
                encoded: format!("minecraft:{log}|axis={axis}"),
                archetype: BlockArchetype::Normal,
            }
        }
        "snow_layer" => {
            let layers = (data_value & 0b0111) + 1;
            Block {
                name: ns("snow"),
                encoded: format!("minecraft:snow|layers={layers}"),
                archetype: BlockArchetype::Normal,
            }
        }
        "stained_hardened_clay" => {
            let col = match data_value & 0b1111 {
                0 => "white",
                1 => "orange",
                2 => "magenta",
                3 => "light_blue",
                4 => "yellow",
                5 => "lime",
                6 => "pink",
                7 => "gray",
                8 => "light_gray",
                9 => "cyan",
                10 => "purple",
                11 => "blue",
                12 => "brown",
                13 => "green",
                14 => "red",
                15 => "black",
                _ => unreachable!(),
            };
            Block {
                name: format!("minecraft:{col}_terracotta"),
                encoded: format!("minecraft:{col}_terracotta|"),
                archetype: BlockArchetype::Normal,
            }
        }
        "hardened_clay" => Block {
            name: ns("terracotta"),
            encoded,
            archetype: BlockArchetype::Normal,
        },
        "tallgrass" => Block {
            name: ns("tall_grass"),
            encoded,
            archetype: BlockArchetype::Normal,
        },
        "waterlily" => Block {
            name: ns("lily_pad"),
            encoded,
            archetype: BlockArchetype::Normal,
        },
        _ => Block {
            name: ns(block_name),
            encoded,
            archetype: BlockArchetype::Normal,
        },
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
        36 => "piston_extension",
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
        104 => "pumpkin_stem",
        105 => "melon_stem",
        106 => "vine",
        107 => "fence_gate",
        108 => "brick_stairs",
        109 => "stone_brick_stairs",
        110 => "mycelium",
        111 => "waterlily",
        112 => "nether_brick",
        113 => "nether_brick_fence",
        114 => "nether_brick_stairs",
        115 => "nether_wart",
        116 => "enchanting_table",
        117 => "brewing_stand",
        118 => "cauldron",
        119 => "end_portal",
        120 => "end_portal_frame",
        121 => "end_stone",
        122 => "dragon_egg",
        123 => "redstone_lamp",
        124 => "lit_redstone_lamp",
        125 => "double_wooden_slab",
        126 => "wooden_slab",
        127 => "cocoa",
        128 => "sandstone_stairs",
        129 => "emerald_ore",
        130 => "ender_chest",
        131 => "tripwire_hook",
        132 => "tripwire",
        133 => "emerald_block",
        134 => "spruce_stairs",
        135 => "birch_stairs",
        136 => "jungle_stairs",
        137 => "command_block",
        138 => "beacon",
        139 => "cobblestone_wall",
        140 => "flower_pot",
        141 => "carrots",
        142 => "potatoes",
        143 => "oak_button",
        144 => "skull",
        145 => "anvil",
        146 => "trapped_chest",
        147 => "light_weighted_pressure_plate",
        148 => "heavy_weighted_pressure_plate",
        149 => "unpowered_comparator",
        150 => "powered_comparator",
        151 => "daylight_detector",
        152 => "redstone_block",
        153 => "quartz_ore",
        154 => "hopper",
        155 => "quartz_block",
        156 => "quartz_stairs",
        157 => "activator_rail",
        158 => "dropper",
        159 => "stained_hardened_clay",
        160 => "stained_glass_pane",
        161 => "leaves2",
        162 => "log2",
        163 => "acacia_stairs",
        164 => "dark_oak_stairs",
        165 => "slime",
        166 => "barrier",
        167 => "iron_trapdoor",
        168 => "prismarine",
        169 => "sea_lantern",
        170 => "hay_block",
        171 => "carpet",
        172 => "hardened_clay",
        173 => "coal_block",
        174 => "packed_ice",
        175 => "double_plant",
        176 => "standing_banner",
        177 => "wall_banner",
        178 => "daylight_detector_inverted",
        179 => "red_sandstone",
        180 => "red_sandstone_stairs",
        181 => "double_stone_slab2",
        182 => "stone_slab2",
        183 => "spruce_fence_gate",
        184 => "birch_fence_gate",
        185 => "jungle_fence_gate",
        186 => "dark_oak_fence_gate",
        187 => "acacia_fence_gate",
        188 => "spruce_fence",
        189 => "birch_fence",
        190 => "jungle_fence",
        191 => "dark_oak_fence",
        192 => "acacia_fence",
        193 => "spruce_door",
        194 => "birch_door",
        195 => "jungle_door",
        196 => "acacia_door",
        197 => "dark_oak_door",
        198 => "end_rod",
        199 => "chorus_plant",
        200 => "chorus_flower",
        201 => "purpur_block",
        202 => "purpur_pillar",
        203 => "purpur_stairs",
        204 => "purpur_double_slab",
        205 => "purpur_slab",
        206 => "end_bricks",
        207 => "beetroots",
        208 => "grass_path",
        209 => "end_gateway",
        210 => "repeating_command_block",
        211 => "chain_command_block",
        212 => "frosted_ice",
        213 => "magma",
        214 => "nether_wart_block",
        215 => "red_nether_brick",
        216 => "bone_block",
        217 => "structure_void",
        218 => "observer",
        219 => "white_shulker_box",
        220 => "orange_shulker_box",
        221 => "magenta_shulker_box",
        222 => "light_blue_shulker_box",
        223 => "yellow_shulker_box",
        224 => "lime_shulker_box",
        225 => "pink_shulker_box",
        226 => "gray_shulker_box",
        227 => "silver_shulker_box",
        228 => "cyan_shulker_box",
        229 => "purple_shulker_box",
        230 => "blue_shulker_box",
        231 => "brown_shulker_box",
        232 => "green_shulker_box",
        233 => "red_shulker_box",
        234 => "black_shulker_box",
        235 => "white_glazed_terracotta",
        236 => "orange_glazed_terracotta",
        237 => "magenta_glazed_terracotta",
        238 => "light_blue_glazed_terracotta",
        239 => "yellow_glazed_terracotta",
        240 => "lime_glazed_terracotta",
        241 => "pink_glazed_terracotta",
        242 => "gray_glazed_terracotta",
        243 => "silver_glazed_terracotta",
        244 => "cyan_glazed_terracotta",
        245 => "purple_glazed_terracotta",
        246 => "blue_glazed_terracotta",
        247 => "brown_glazed_terracotta",
        248 => "green_glazed_terracotta",
        249 => "red_glazed_terracotta",
        250 => "black_glazed_terracotta",
        251 => "concrete",
        252 => "concrete_powder",
        253 => "",
        254 => "",
        255 => "structure_block",
    }
}
