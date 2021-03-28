use crate::{Block, CCoord, Chunk, RCoord, Region, MIN_Y};

use super::biome::Biome;

pub type Rgba = [u8; 4];

/// ChunkRender objects can render a given chunk. What they render to is
/// entirely up to the implementation.
pub trait ChunkRender {
    /// Draw the given chunk.
    fn draw(&mut self, x: CCoord, z: CCoord, chunk: &dyn Chunk);

    /// Draw the invalid chunk. This means that the chunk was not of an expected
    /// form and couldn't be deserialized into a chunk object.
    fn draw_invalid(&mut self, x: CCoord, z: CCoord);
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

pub fn parse_region<F: ChunkRender + ?Sized>(
    region: &dyn Region,
    draw_to: &mut F,
) -> DrawResult<()> {
    for z in 0isize..32 {
        for x in 0isize..32 {
            let (x, z) = (CCoord(x), CCoord(z));
            region.chunk(x, z).map(|chunk| draw_to.draw(x, z, &*chunk));
        }
    }

    Ok(())
}

pub struct RegionBlockDrawer<'a, P: Palette + ?Sized> {
    pub map: RegionMap<Rgba>,
    pub palette: &'a P,
    pub processed_chunks: usize,
    pub painted_pixels: usize,
}

impl<'a, P: Palette + ?Sized> RegionBlockDrawer<'a, P> {
    pub fn new(map: RegionMap<Rgba>, palette: &'a P) -> Self {
        Self {
            map,
            palette,
            processed_chunks: 0,
            painted_pixels: 0,
        }
    }
}

impl<'a, P: Palette + ?Sized> IntoMap for RegionBlockDrawer<'a, P> {
    fn into_map(self) -> RegionMap<Rgba> {
        self.map
    }
}

impl<'a, P: Palette + ?Sized> ChunkRender for RegionBlockDrawer<'a, P> {
    fn draw(&mut self, x: CCoord, z: CCoord, chunk: &dyn Chunk) {
        let data = self.map.chunk_mut(x, z);
        self.processed_chunks += 1;

        if chunk.status() != "full" && chunk.status() != "spawn" {
            // Chunks that have been fully generated will have a 'full' status.
            // Skip chunks that don't; the way they render is unpredictable.
            return;
        }

        for z in 0..16 {
            for x in 0..16 {
                let height = chunk.surface_height(x, z);

                let height = if height == MIN_Y { MIN_Y } else { height - 1 }; // -1 because we want the block below the air.
                let biome = chunk.biome(x, height, z);
                let block = chunk.block(x, height, z).unwrap(); // Block should definitely exist as we just figured out the height of it.

                let colour = self.palette.pick(&block, biome);

                let pixel = &mut data[z * 16 + x];
                *pixel = colour;
                self.painted_pixels += 1;
            }
        }
    }

    fn draw_invalid(&mut self, x: CCoord, z: CCoord) {
        let data = self.map.chunk_mut(x, z);

        // Draw a red cross over the chunk
        for z in 0..16isize {
            for x in 0..16isize {
                let pixel = &mut data[z as usize * 16 + x as usize];

                *pixel = if (x - z).abs() < 3 || (x - (16 - z)).abs() < 3 {
                    [255, 0, 0, 255]
                } else {
                    [0, 0, 0, 0]
                }
            }
        }
    }
}
