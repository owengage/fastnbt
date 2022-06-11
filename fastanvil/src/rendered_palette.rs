use log::debug;

use crate::{biome::Biome, Block, Palette, Rgba, SNOW_BLOCK};

pub struct RenderedPalette {
    pub blockstates: std::collections::HashMap<String, Rgba>,
    pub grass: image::RgbaImage,
    pub foliage: image::RgbaImage,
}

impl RenderedPalette {
    fn pick_grass(&self, b: Option<Biome>) -> Rgba {
        b.map(|b| {
            let climate = b.climate();
            let t = climate.temperature.min(1.).max(0.);
            let r = climate.rainfall.min(1.).max(0.) * t;

            let t = 255 - (t * 255.).ceil() as u32;
            let r = 255 - (r * 255.).ceil() as u32;

            self.grass.get_pixel(t, r).0
        })
        .unwrap_or([255, 0, 0, 0])
    }

    fn pick_foliage(&self, b: Option<Biome>) -> Rgba {
        b.map(|b| {
            let climate = b.climate();
            let t = climate.temperature.min(1.).max(0.);
            let r = climate.rainfall.min(1.).max(0.) * t;

            let t = 255 - (t * 255.).ceil() as u32;
            let r = 255 - (r * 255.).ceil() as u32;

            self.foliage.get_pixel(t, r).0
        })
        .unwrap_or([255, 0, 0, 0])
    }

    fn pick_water(&self, b: Option<Biome>) -> Rgba {
        use Biome::*;
        b.map(|b| match b {
            Swamp => [0x61, 0x7B, 0x64, 255],
            River => [0x3F, 0x76, 0xE4, 255],
            Ocean => [0x3F, 0x76, 0xE4, 255],
            LukewarmOcean => [0x45, 0xAD, 0xF2, 255],
            WarmOcean => [0x43, 0xD5, 0xEE, 255],
            ColdOcean => [0x3D, 0x57, 0xD6, 255],
            FrozenRiver => [0x39, 0x38, 0xC9, 255],
            FrozenOcean => [0x39, 0x38, 0xC9, 255],
            _ => [0x3f, 0x76, 0xe4, 255],
        })
        .unwrap_or([0x3f, 0x76, 0xe4, 255])
    }
}

impl Palette for RenderedPalette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba {
        let missing_colour = [255, 0, 255, 255];

        // A bunch of blocks in the game seem to be special cased outside of the
        // blockstate/model mechanism. For example leaves get coloured based on
        // the tree type and the biome type, but this is not encoded in the
        // blockstate or the model.
        //
        // This means we have to do a bunch of complex conditional logic in one
        // of the most called functions. Yuck.
        if let Some(id) = block.name().strip_prefix("minecraft:") {
            match id {
                "grass" | "tall_grass" | "vine" | "fern" | "large_fern" => {
                    return self.pick_grass(biome);
                }
                "grass_block" => {
                    if block.snowy() {
                        return self.pick(&SNOW_BLOCK, biome);
                    } else {
                        return self.pick_grass(biome);
                    };
                }
                "water" | "bubble_column" => return self.pick_water(biome),
                "oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves"
                | "mangrove_leaves" => return self.pick_foliage(biome),
                "birch_leaves" => {
                    return [0x80, 0xa7, 0x55, 255]; // game hardcodes this
                }
                "spruce_leaves" => {
                    return [0x61, 0x99, 0x61, 255]; // game hardcodes this
                }
                // Kelp and seagrass don't look like much from the top as
                // they're flat. Maybe in future hard code a green tint to make
                // it show up?
                "kelp" | "kelp_plant" | "seagrass" | "tall_seagrass" => {
                    return self.pick_water(biome);
                }
                "snow" => {
                    return self.pick(&SNOW_BLOCK, biome);
                }
                // Occurs a lot for the end, as layer 0 will be air in the void.
                // Rendering it black makes sense in the end, but might look
                // weird if it ends up elsewhere.
                "air" => {
                    return [0, 0, 0, 255];
                }
                "cave_air" => {
                    return [255, 0, 0, 255]; // when does this happen??
                }
                // Otherwise fall through to the general mechanism.
                _ => {}
            }
        }

        let col = self
            .blockstates
            .get(block.encoded_description())
            .or_else(|| self.blockstates.get(block.name()));

        match col {
            Some(c) => *c,
            None => {
                debug!("could not draw {}", block.name());
                debug!("description {}", block.encoded_description());

                missing_colour
            }
        }
    }
}
