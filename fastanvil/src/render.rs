use std::{
    cmp::Ordering,
    io::{Read, Seek, Write},
};

use crate::{
    Block, BlockArchetype, CCoord, Chunk, HeightMode, JavaChunk, LoaderError, LoaderResult, RCoord,
    RegionLoader,
};

use super::biome::Biome;

pub type Rgba = [u8; 4];

/// Palette can be used to take a block description to produce a colour that it
/// should render to.
pub trait Palette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba;
}

pub trait Renderer {
    fn render<C: Chunk + ?Sized>(&self, chunk: &C, north: Option<&C>) -> [Rgba; 16 * 16];
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

    /// Drill for colour. Starting at y_start, make way down the column until we
    /// have an opaque colour to return. This tackles things like transparency.
    fn drill_for_colour<C: Chunk + ?Sized>(
        &self,
        x: usize,
        y_start: isize,
        z: usize,
        chunk: &C,
        y_min: isize,
    ) -> Rgba {
        let mut y = y_start;
        let mut colour = [0, 0, 0, 0];

        while colour[3] != 255 && y >= y_min {
            let current_biome = chunk.biome(x, y, z);
            let current_block = chunk.block(x, y, z);

            if let Some(current_block) = current_block {
                match current_block.archetype {
                    BlockArchetype::Airy => {
                        y -= 1;
                    }
                    // TODO: Can potentially optimize this for ocean floor using
                    // heightmaps.
                    BlockArchetype::Watery => {
                        let mut block_colour = self.palette.pick(current_block, current_biome);
                        let water_depth = water_depth(x, y, z, chunk, y_min);
                        let alpha = water_depth_to_alpha(water_depth);

                        block_colour[3] = alpha;

                        colour = a_over_b_colour(colour, block_colour);
                        y -= water_depth;
                    }
                    _ => {
                        let block_colour = self.palette.pick(current_block, current_biome);
                        colour = a_over_b_colour(colour, block_colour);
                        y -= 1;
                    }
                }
            } else {
                return colour;
            }
        }

        colour
    }
}

impl<'a, P: Palette> Renderer for TopShadeRenderer<'a, P> {
    fn render<C: Chunk + ?Sized>(&self, chunk: &C, north: Option<&C>) -> [Rgba; 16 * 16] {
        let mut data = [[0, 0, 0, 0]; 16 * 16];

        let status = chunk.status();
        const OK_STATUSES: [&str; 8] = [
            "full",
            "spawn",
            "postprocessed",
            "fullchunk",
            "minecraft:full",
            "minecraft:spawn",
            "minecraft:postprocessed",
            "minecraft:fullchunk",
        ];
        if !OK_STATUSES.contains(&status.as_str()) {
            // Chunks that have been fully generated will have a 'full' status.
            // Skip chunks that don't; the way they render is unpredictable.
            return data;
        }

        let y_range = chunk.y_range();

        for z in 0..16 {
            for x in 0..16 {
                let air_height = chunk.surface_height(x, z, self.height_mode);
                let block_height = (air_height - 1).max(y_range.start);

                let colour = self.drill_for_colour(x, block_height, z, chunk, y_range.start);

                let north_air_height = match z {
                    // if top of chunk, get height from the chunk above.
                    0 => north
                        .map(|c| c.surface_height(x, 15, self.height_mode))
                        .unwrap_or(block_height),
                    z => chunk.surface_height(x, z - 1, self.height_mode),
                };
                let colour = top_shade_colour(colour, air_height, north_air_height);

                data[z * 16 + x] = colour;
            }
        }

        data
    }
}

/// Convert `water_depth` meters of water to an approximate opacity
fn water_depth_to_alpha(water_depth: isize) -> u8 {
    // Water will absorb a fraction of the light per unit depth. So if we say
    // that every metre of water absorbs half the light going through it, then 2
    // metres would absorb 3/4, 3 metres would absorb 7/8 etc.
    //
    // Since RGB is not linear, we can make a very rough approximation of this
    // fractional behavior with a linear equation in RBG space. This pretends
    // water absorbs quadratically x^2, rather than exponentially e^x.
    //
    // We put a lower limit to make rivers and swamps still have water show up
    // well, and an upper limit so that very deep ocean has a tiny bit of
    // transparency still.
    //
    // This is pretty rather than accurate.

    (180 + 2 * water_depth).min(250) as u8
}

fn water_depth<C: Chunk + ?Sized>(
    x: usize,
    mut y: isize,
    z: usize,
    chunk: &C,
    y_min: isize,
) -> isize {
    let mut depth = 1;
    while y > y_min {
        let block = match chunk.block(x, y, z) {
            Some(b) => b,
            None => return depth,
        };

        if block.archetype == BlockArchetype::Watery {
            depth += 1;
        } else {
            return depth;
        }
        y -= 1;
    }
    depth
}

/// Merge two potentially transparent colours, A and B, into one as if colour A
/// was laid on top of colour B.
///
/// See https://en.wikipedia.org/wiki/Alpha_compositing
fn a_over_b_colour(colour: [u8; 4], below_colour: [u8; 4]) -> [u8; 4] {
    let linear = |c: u8| (((c as usize).pow(2)) as f32) / ((255 * 255) as f32);
    let colour = colour.map(linear);
    let below_colour = below_colour.map(linear);

    let over_component = |ca: f32, aa: f32, cb: f32, ab: f32| {
        let a_out = aa + ab * (1. - aa);
        let linear_out = (ca * aa + cb * ab * (1. - aa)) / a_out;
        (linear_out * 255. * 255.).sqrt() as u8
    };

    let over_alpha = |aa: f32, ab: f32| {
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

pub fn render_region<S>(
    x: RCoord,
    z: RCoord,
    loader: &dyn RegionLoader<S>,
    renderer: &impl Renderer,
) -> LoaderResult<Option<RegionMap<Rgba>>>
where
    S: Seek + Read + Write,
{
    let mut map = RegionMap::new(x, z, [0u8; 4]);

    let mut region = match loader.region(x, z)? {
        Some(r) => r,
        None => return Ok(None),
    };

    let mut cache: [Option<JavaChunk>; 32] = Default::default();

    // Cache the last row of chunks from the above region to allow top-shading
    // on region boundaries.
    if let Some(mut r) = loader.region(x, RCoord(z.0 - 1))? {
        for (x, entry) in cache.iter_mut().enumerate() {
            *entry = r
                .read_chunk(x, 31)
                .ok()
                .flatten()
                .and_then(|b| JavaChunk::from_bytes(&b).ok())
        }
    }

    for z in 0usize..32 {
        for (x, cache) in cache.iter_mut().enumerate() {
            let data = map.chunk_mut(CCoord(x as isize), CCoord(z as isize));

            let chunk_data = region
                .read_chunk(x, z)
                .map_err(|e| LoaderError(e.to_string()))?;
            let chunk_data = match chunk_data {
                Some(data) => data,
                None => {
                    // If there's no chunk here, we still need to set the cache
                    // otherwise the chunks below this will top-shade with an
                    // incorrect chunk.
                    *cache = None;
                    continue;
                }
            };

            let chunk =
                JavaChunk::from_bytes(&chunk_data).map_err(|e| LoaderError(e.to_string()))?;

            // Get the chunk at the same x coordinate from the cache. This
            // should be the chunk that is directly above the current. We
            // know this because once we have processed this chunk we put it
            // in the cache in the same place. So the next time we get the
            // current one will be when we're processing directly below us.
            //
            // Thanks to the default None value this works fine for the
            // first row or for any missing chunks.
            let north = cache.as_ref();

            let res = renderer.render(&chunk, north);
            *cache = Some(chunk);

            data[..].clone_from_slice(&res);
        }
    }

    Ok(Some(map))
}

/// Apply top-shading to the given colour based on the relative height of the
/// block above it. Darker if the above block is taller, and lighter if it's
/// smaller.
///
/// Technically this function darkens colours, but this is also how Minecraft
/// itself shades maps.
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
