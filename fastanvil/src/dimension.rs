use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Display, rc::Rc};

use crate::{biome::Biome, Block};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RCoord(pub isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CCoord(pub isize);

pub trait Chunk {
    fn status(&self) -> String;

    /// Get the height of the first air-like block above something not air-like.
    /// Will panic if given x/z coordinates outside of 0..16.
    fn surface_height(&self, x: usize, z: usize) -> isize;

    /// Get the biome of the given coordinate. A biome may not exist if the
    /// section of the chunk accessed is not present. For example,
    /// trying to access the block at height 1234 would return None.
    fn biome(&self, x: usize, y: isize, z: usize) -> Option<Biome>;

    /// Get the block at the given coordinates. A block may not exist if the
    /// section of the chunk accessed is not present. For example,
    /// trying to access the block at height 1234 would return None.
    fn block(&self, x: usize, y: isize, z: usize) -> Option<Block>;
}

pub trait Region {
    fn chunk(&self, x: CCoord, z: CCoord) -> Option<Box<dyn Chunk>>;
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
pub trait RegionLoader {
    fn region(&self, x: RCoord, z: RCoord) -> Option<Box<dyn Region>>;
    fn list(&self) -> LoaderResult<Vec<(RCoord, RCoord)>>;
}

/// Dimension provides a cache on top of a RegionLoader.
pub struct Dimension {
    loader: Box<dyn RegionLoader>,
    regions: RefCell<HashMap<(RCoord, RCoord), Rc<dyn Region>>>,
}

impl Dimension {
    pub fn new(loader: Box<dyn RegionLoader>) -> Self {
        Self {
            loader,
            regions: Default::default(),
        }
    }

    /// Get a region, maybe from Dimension's internal cache.
    pub fn region(&self, x: RCoord, z: RCoord) -> Option<Rc<dyn Region>> {
        let mut cache = self.regions.borrow_mut();

        cache.get(&(x, z)).map(|r| Rc::clone(r)).or_else(|| {
            let r = Rc::from(self.loader.region(x, z)?);
            cache.insert((x, z), Rc::clone(&r));
            Some(r)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct DummyRegion;
    impl Region for DummyRegion {
        fn chunk(&self, _x: CCoord, _z: CCoord) -> Option<Box<dyn Chunk>> {
            unimplemented!()
        }
    }

    struct DummyLoader;
    impl RegionLoader for DummyLoader {
        fn region(&self, x: RCoord, z: RCoord) -> Option<Box<dyn Region>> {
            Some(Box::new(DummyRegion))
        }

        fn list(&self) -> LoaderResult<Vec<(RCoord, RCoord)>> {
            todo!()
        }
    }

    #[test]
    fn get_region() {
        let d = Dimension::new(Box::new(DummyLoader));
        let _region = d.region(RCoord(0), RCoord(0));
    }

    // TODO: Get a chunk!
}
