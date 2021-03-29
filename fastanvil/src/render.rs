use std::cmp::Ordering;

use crate::{Block, CCoord, Chunk, Dimension, RCoord, MIN_Y};

use super::biome::Biome;

pub type Rgba = [u8; 4];

pub struct TopShadeRenderer<'a, P: Palette> {
    palette: &'a P,
}

impl<'a, P: Palette> TopShadeRenderer<'a, P> {
    pub fn new(palette: &'a P) -> Self {
        Self { palette }
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
                let height = chunk.surface_height(x, z);

                let shade_height = match z {
                    0 => above.map(|c| c.surface_height(x, 15)).unwrap_or(height),
                    z => chunk.surface_height(x, z - 1),
                };

                let shade = match height.cmp(&shade_height) {
                    Ordering::Less => 180usize,
                    Ordering::Equal => 220,
                    Ordering::Greater => 255,
                };

                let height = if height == MIN_Y { MIN_Y } else { height - 1 }; // -1 because we want the block below the air.
                let biome = chunk.biome(x, height, z);
                let block = chunk.block(x, height, z).unwrap(); // Block should definitely exist as we just figured out the height of it.

                let mut colour = self.palette.pick(&block, biome);

                colour = [
                    (colour[0] as usize * shade / 255) as u8,
                    (colour[1] as usize * shade / 255) as u8,
                    (colour[2] as usize * shade / 255) as u8,
                    colour[3],
                ];

                let pixel = &mut data[z * 16 + x];
                *pixel = colour;
            }
        }

        data
    }
}

/// Palette can be used to take a block description to produce a colour that it
/// should render to.
pub trait Palette {
    fn pick(&self, block: &Block, biome: Option<Biome>) -> Rgba;
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

            // TODO: Must be a better way to do this.
            chunk_data.map(|d| {
                for i in 0..data.len() {
                    data[i] = d[i];
                }
            });
        }
    }

    map
}
