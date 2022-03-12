use serde::de::DeserializeOwned;

use crate::{Chunk, JavaChunk, LoaderError};
use crate::{LoaderResult, RegionBuffer};
use crate::{RCoord, Region, RegionLoader};
use std::marker::PhantomData;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct RegionFileLoader {
    region_dir: PathBuf,
    _d: PhantomData<JavaChunk>,
}

impl RegionFileLoader {
    pub fn new(region_dir: PathBuf) -> Self {
        Self {
            region_dir,
            _d: PhantomData,
        }
    }
}

impl RegionLoader for RegionFileLoader {
    fn region(&self, x: RCoord, z: RCoord) -> Option<Box<dyn Region>> {
        let path = self.region_dir.join(format!("r.{}.{}.mca", x.0, z.0));
        let file = std::fs::File::open(path).ok()?;
        let region = RegionBuffer::new(file);

        Some(Box::new(region))
    }

    fn list(&self) -> LoaderResult<Vec<(RCoord, RCoord)>> {
        let paths = std::fs::read_dir(&self.region_dir).map_err(|e| LoaderError(e.to_string()))?;

        let paths = paths
            .into_iter()
            .filter_map(|path| path.ok())
            .map(|path| path.path())
            .filter(|path| path.is_file())
            .filter(|path| {
                let ext = path.extension();
                ext.is_some() && ext.unwrap() == "mca"
            })
            .filter(|path| fs::metadata(path).unwrap().len() > 0)
            .filter_map(|p| coords_from_region(&p))
            .collect();

        Ok(paths)
    }
}

fn coords_from_region(region: &Path) -> Option<(RCoord, RCoord)> {
    let filename = region.file_name()?.to_str()?;
    let mut parts = filename.split('.').skip(1);
    let x = parts.next()?.parse::<isize>().ok()?;
    let z = parts.next()?.parse::<isize>().ok()?;
    Some((RCoord(x), RCoord(z)))
}
