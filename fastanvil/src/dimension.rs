use std::{cell::RefCell, collections::HashMap, error::Error, fmt::Display, ops::Range, rc::Rc};

use crate::{biome::Biome, Block};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RCoord(pub isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CCoord(pub isize);

#[derive(Clone, Copy)]
pub enum HeightMode {
    Trust,     // trust height maps from chunk data
    Calculate, // calculate height maps manually, much slower.
}

pub trait Chunk {
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

pub trait Region<C: Chunk> {
    /// Load the chunk at the given chunk coordinates, ie 0..32 for x and z.
    /// Implmentations do not need to be concerned with caching chunks they have
    /// loaded, this will be handled by the types using the region.
    fn chunk(&self, x: CCoord, z: CCoord) -> Option<C>;
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
pub trait RegionLoader<C: Chunk> {
    /// Get a particular region. Returns None if region does not exist.
    fn region(&self, x: RCoord, z: RCoord) -> Option<Box<dyn Region<C>>>;

    /// List the regions that this loader can return. Implmentations need to
    /// provide this so that callers can efficiently find regions to process.
    fn list(&self) -> LoaderResult<Vec<(RCoord, RCoord)>>;
}

/// Dimension provides a cache on top of a RegionLoader.
pub struct Dimension<C: Chunk> {
    loader: Box<dyn RegionLoader<C>>,
    regions: RefCell<HashMap<(RCoord, RCoord), Rc<dyn Region<C>>>>,
}

impl<C: Chunk> Dimension<C> {
    pub fn new(loader: Box<dyn RegionLoader<C>>) -> Self {
        Self {
            loader,
            regions: Default::default(),
        }
    }

    /// Get a region, maybe from Dimension's internal cache.
    pub fn region(&self, x: RCoord, z: RCoord) -> Option<Rc<dyn Region<C>>> {
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
    use std::marker::PhantomData;

    use super::*;

    struct DummyRegion<C: Chunk>(PhantomData<C>);

    impl<C: Chunk> Region<C> for DummyRegion<C> {
        fn chunk(&self, _x: CCoord, _z: CCoord) -> Option<C> {
            unimplemented!()
        }
    }

    struct DummyLoader<C: Chunk>(PhantomData<C>);

    impl<C: Chunk + 'static> RegionLoader<C> for DummyLoader<C> {
        fn region(&self, _x: RCoord, _z: RCoord) -> Option<Box<dyn Region<C>>> {
            Some(Box::new(DummyRegion::<C>(PhantomData)))
        }

        fn list(&self) -> LoaderResult<Vec<(RCoord, RCoord)>> {
            todo!()
        }
    }
}
