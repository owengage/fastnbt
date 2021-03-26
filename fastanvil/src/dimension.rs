use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RCoord(pub isize);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct CCoord(pub isize);

pub trait Chunk {}

pub trait Region {
    fn chunk(&self, x: CCoord, z: CCoord) -> Option<Box<dyn Chunk>>;
}

/// RegionLoader implmentations provide access to regions. They do not need to
/// be concerned with caching, just providing an object that implements Region.
///
/// An example implementation could be loading a region file from a local disk,
/// or perhaps a WASM version loading from a file buffer in the browser.
pub trait RegionLoader {
    fn region(&self, x: RCoord, z: RCoord) -> Option<Box<dyn Region>>;
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
    }

    #[test]
    fn get_region() {
        let d = Dimension::new(Box::new(DummyLoader));
        let _region = d.region(RCoord(0), RCoord(0));
    }

    // TODO: Get a chunk!
}
