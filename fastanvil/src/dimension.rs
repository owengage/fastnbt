use std::io::{Read, Seek, Write};
use std::{error::Error, fmt::Display, ops::Range};

use serde::{Deserialize, Serialize};

use crate::Region;
use crate::{biome::Biome, Block};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RCoord(pub isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CCoord(pub isize);

#[derive(Debug, Clone, Copy)]
pub enum HeightMode {
    Trust,     // trust height maps from chunk data
    Calculate, // calculate height maps manually, much slower.
}

pub trait Chunk: Send + Sync {
    // Status of the chunk. Typically anything except 'full' means the chunk
    // hasn't been fully generated yet. We use this to skip chunks on map edges
    // that haven't been fully generated yet.
    fn status(&self) -> String;

    /// Get the height of the first air-like block above something not air-like.
    /// Will panic if given x/z coordinates outside of 0..16.
    fn surface_height(&self, x: usize, z: usize, mode: HeightMode) -> isize;

    /// Get the biome of the given coordinate. A biome may not exist if the
    /// section of the chunk accessed is not present. For example,
    /// trying to access the block at height 1234 would return None.
    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome>;

    /// Get the block at the given coordinates. A block may not exist if the
    /// section of the chunk accessed is not present. For example,
    /// trying to access the block at height 1234 would return None.
    fn block(&self, x: usize, y: isize, z: usize) -> Option<&Block>;

    /// Get the range of Y values that are valid for this chunk.
    fn y_range(&self) -> Range<isize>;
}

#[derive(Debug)]
pub struct LoaderError(pub(crate) String);

pub type LoaderResult<T> = std::result::Result<T, LoaderError>;

impl Error for LoaderError {}

impl Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// RegionLoader implmentations provide access to regions. They do not need to
/// be concerned with caching, just providing an object that implements Region.
///
/// An example implementation could be loading a region file from a local disk,
/// or perhaps a WASM version loading from a file buffer in the browser.
pub trait RegionLoader<S>
where
    S: Seek + Read + Write,
{
    /// Get a particular region. Returns Ok(None) if region does not exist.
    fn region(&self, x: RCoord, z: RCoord) -> LoaderResult<Option<Region<S>>>;

    /// List the regions that this loader can return. Implmentations need to
    /// provide this so that callers can efficiently find regions to process.
    fn list(&self) -> LoaderResult<Vec<(RCoord, RCoord)>>;
}
