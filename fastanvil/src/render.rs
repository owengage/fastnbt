use std::cmp::Ordering;

use crate::{Block, CCoord, Chunk, Dimension, HeightMode, RCoord, MIN_Y};

use super::biome::Biome;

pub type Rgba = [u8; 4];

/// Palette can be used to take a block description to produce a colour that it
/// should render to.
pub trait Palette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba;
}

pub struct TopShadeRenderer<'a, P: Palette> {
    palette: &'a P,
    height_mode: HeightMode,
}

impl<'a, P: Palette> TopShadeRenderer<'a, P> {
    pub fn new(palette: &'a P, mode: HeightMode) -> Self {
        Self {
            palette,
            height_mode: mode,
        }
    }

    fn render(&self, chunk: &dyn Chunk, above: Option<&dyn Chunk>) -> [Rgba; 16 * 16] {
        let mut data = [[0, 0, 0, 0]; 16 * 16];

        if chunk.status() != "full" && chunk.status() != "spawn" {
            // Chunks that have been fully generated will have a 'full' status.
            // Skip chunks that don't; the way they render is unpredictable.
            return data;
        }

        for z in 0..16 {
            for x in 0..16 {
                let height = chunk.surface_height(x, z, self.height_mode);

                let height = if height == MIN_Y { MIN_Y } else { height - 1 }; // -1 because we want the block below the air.
                let biome = chunk.biome(x, height, z);
                let block = chunk.block(x, height, z);

                // TODO: Under what circumstances does the block not exist?
                // Feels like it always should. Seems to be related to a section
                // of the chunk existing, but having an empty palette and block
                // states. Does not fall on any decernable boundary.
                let mut colour = match block {
                    Some(ref block) => self.palette.pick(&block, biome),
                    None => [255, 0, 255, 255],
                };

                // Now that we have the colour of the top layer, we can check
                // the transparency of it. If it is transparent we can look for
                // a block below it to get a better render.

                let transparency_depth = 3;
                let mut depth = 0;
                let mut last_height = height;

                while colour[3] != 255 && depth <= transparency_depth {
                    let (below_block, below_height) = block_below(x, last_height, z, chunk);
                    if let Some(below_block) = below_block {
                        let below_colour = self.palette.pick(&below_block, biome); // TODO: Biome might have changed.
                        colour = a_over_b_colour(colour, below_colour);
                    }

                    depth = depth + 1;
                    last_height = below_height;
                }

                // fudge alpha to max so final pixel has no transparency.
                colour[3] = 255;

                let shade_height = match z {
                    // if top of chunk, get height from the chunk above.
                    0 => above
                        .map(|c| c.surface_height(x, 15, self.height_mode))
                        .unwrap_or(height),
                    z => chunk.surface_height(x, z - 1, self.height_mode),
                };
                let colour = top_shade_colour(colour, height, shade_height);

                data[z * 16 + x] = colour;
            }
        }

        data
    }
}

fn block_below(x: usize, mut y: isize, z: usize, chunk: &dyn Chunk) -> (Option<Block>, isize) {
    while y >= MIN_Y {
        y = y - 1;
        let block = chunk.block(x, y - 1, z);

        if block.is_none() {
            continue;
        }

        if !["minecraft:air", "minecraft:cave_air"]
            .as_ref()
            .contains(&block.as_ref().unwrap().name.as_str())
        {
            return (block, y);
        }
    }

    (None, y)
}

fn a_over_b_colour(colour: [u8; 4], below_colour: [u8; 4]) -> [u8; 4] {
    let linear = |c: u8| (((c as usize).pow(2)) as f32) / ((255 * 255) as f32);

    let over_component = |ca: u8, aa: u8, cb: u8, ab: u8| {
        // See https://en.wikipedia.org/wiki/Alpha_compositing
        let ca = linear(ca);
        let cb = linear(cb);
        let aa = linear(aa);
        let ab = linear(ab);
        let a_out = aa + ab * (1. - aa);
        let linear_out = (ca * aa + cb * ab * (1. - aa)) / a_out;
        (linear_out * 255. * 255.).sqrt() as u8
    };

    let over_alpha = |aa: u8, ab: u8| {
        // See https://en.wikipedia.org/wiki/Alpha_compositing
        let aa = linear(aa);
        let ab = linear(ab);
        let a_out = aa + ab * (1. - aa);
        (a_out * 255. * 255.).sqrt() as u8
    };

    [
        over_component(colour[0], colour[3], below_colour[0], below_colour[3]),
        over_component(colour[1], colour[3], below_colour[1], below_colour[3]),
        over_component(colour[2], colour[3], below_colour[2], below_colour[3]),
        over_alpha(colour[3], below_colour[3]),
    ]
}

pub trait IntoMap {
    fn into_map(self) -> RegionMap<Rgba>;
}

pub struct RegionMap<T> {
    pub data: Vec<T>,
    pub x: RCoord,
    pub z: RCoord,
}

impl<T: Clone> RegionMap<T> {
    pub fn new(x: RCoord, z: RCoord, default: T) -> Self {
        let mut data: Vec<T> = Vec::new();
        data.resize(16 * 16 * 32 * 32, default);
        Self { data, x, z }
    }

    pub fn chunk_mut(&mut self, x: CCoord, z: CCoord) -> &mut [T] {
        debug_assert!(x.0 >= 0 && z.0 >= 0);

        let len = 16 * 16;
        let begin = (z.0 * 32 + x.0) as usize * len;
        &mut self.data[begin..begin + len]
    }

    pub fn chunk(&self, x: CCoord, z: CCoord) -> &[T] {
        debug_assert!(x.0 >= 0 && z.0 >= 0);

        let len = 16 * 16;
        let begin = (z.0 * 32 + x.0) as usize * len;
        &self.data[begin..begin + len]
    }
}

pub fn parse_region<P: Palette>(
    x: RCoord,
    z: RCoord,
    dimension: Dimension,
    renderer: TopShadeRenderer<P>,
) -> RegionMap<Rgba> {
    let mut map = RegionMap::new(x, z, [0u8; 4]);

    let region = match dimension.region(x, z) {
        Some(r) => r,
        None => return map,
    };

    let mut cache: [Option<Box<dyn Chunk>>; 32] = Default::default();

    // Cache the last row of chunks from the above region to allow top-shading
    // on region boundaries.
    dimension.region(x, RCoord(z.0 - 1)).map(|r| {
        for x in 0..32 {
            cache[x] = r.chunk(CCoord(x as isize), CCoord(31));
        }
    });

    for z in 0isize..32 {
        for x in 0isize..32 {
            let (x, z) = (CCoord(x), CCoord(z));
            let data = map.chunk_mut(x, z);

            let chunk_data = region.chunk(x, z).map(|chunk| {
                // Get the chunk at the same x coordinate from the cache. This
                // should be the chunk that is directly above the current. We
                // know this because once we have processed this chunk we put it
                // in the cache in the same place. So the next time we get the
                // current one will be when we're processing directly below us.
                //
                // Thanks to the default None value this works fine for the
                // first row or for any missing chunks.
                let above = cache[x.0 as usize].as_ref().map(|c| &**c);
                let res = renderer.render(&*chunk, above);
                cache[x.0 as usize] = Some(chunk);
                res
            });

            chunk_data.map(|d| {
                data[..].clone_from_slice(&d);
            });
        }
    }

    map
}

fn top_shade_colour(colour: Rgba, height: isize, shade_height: isize) -> Rgba {
    let shade = match height.cmp(&shade_height) {
        Ordering::Less => 180usize,
        Ordering::Equal => 220,
        Ordering::Greater => 255,
    };
    [
        (colour[0] as usize * shade / 255) as u8,
        (colour[1] as usize * shade / 255) as u8,
        (colour[2] as usize * shade / 255) as u8,
        colour[3],
    ]
}
