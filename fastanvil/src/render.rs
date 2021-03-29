use std::cmp::Ordering;

use crate::{Block, CCoord, Chunk, RCoord, Region, MIN_Y};

use super::biome::Biome;

pub type Rgba = [u8; 4];

pub trait TopShadeRender {
    /// Render should render the given chunk to RGBA and return it. The chunk
    /// above the current one is provided to allow top-shading. There might not be
    /// a chunk present above the one being rendered.
    fn render(&self, chunk: &dyn Chunk, above: Option<&dyn Chunk>) -> [Rgba; 16 * 16];
}

pub struct TopShadeRenderer<'a, P: Palette> {
    palette: &'a P,
}

impl<'a, P: Palette> TopShadeRenderer<'a, P> {
    pub fn new(palette: &'a P) -> Self {
        Self { palette }
    }
}

impl<'a, P: Palette> TopShadeRender for TopShadeRenderer<'a, P> {
    fn render(&self, chunk: &dyn Chunk, _above: Option<&dyn Chunk>) -> [Rgba; 16 * 16] {
        let mut data = [[0, 0, 0, 0]; 16 * 16];

        if chunk.status() != "full" && chunk.status() != "spawn" {
            // Chunks that have been fully generated will have a 'full' status.
            // Skip chunks that don't; the way they render is unpredictable.
            return data;
        }

        for z in 0..16 {
            for x in 0..16 {
                let height = chunk.surface_height(x, z);
                let shade_height = chunk.surface_height(x, z.saturating_sub(1));
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

#[derive(Debug)]
pub enum DrawError {
    ParseAnvil(super::Error),
    ParseNbt(fastnbt::error::Error),
    IO(std::io::Error),
    MissingHeightMap,
    InvalidPalette,
}

impl From<super::Error> for DrawError {
    fn from(err: super::Error) -> DrawError {
        DrawError::ParseAnvil(err)
    }
}

impl From<fastnbt::error::Error> for DrawError {
    fn from(err: fastnbt::error::Error) -> DrawError {
        DrawError::ParseNbt(err)
    }
}

impl From<std::io::Error> for DrawError {
    fn from(err: std::io::Error) -> Self {
        DrawError::IO(err)
    }
}

pub type DrawResult<T> = std::result::Result<T, DrawError>;

pub fn parse_region<R: TopShadeRender>(
    x: RCoord,
    z: RCoord,
    region: &dyn Region,
    draw_to: R,
) -> RegionMap<Rgba> {
    let mut map = RegionMap::new(x, z, [0u8; 4]);

    for z in 0isize..32 {
        for x in 0isize..32 {
            let (x, z) = (CCoord(x), CCoord(z));
            let data = map.chunk_mut(x, z);

            // TODO: Provide the top chunk to render shading properly!
            // We will need to cache chunks to avoid doing a lot of extra work.
            // This entire function should probably rethought, because
            // eventually we're going to want to access the region above this
            // one. Introduce the Dimension type.
            let chunk_data = region
                .chunk(x, z)
                .map(|chunk| draw_to.render(&*chunk, None));

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
