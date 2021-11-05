use bit_field::BitField;
use fastnbt::LongArray;

use serde::Deserialize;
use std::fmt::Debug;

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct BlockData<T: Debug> {
    inner: DataInner<T>,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct BiomeData<T: Debug> {
    inner: DataInner<T>,
}

#[derive(Deserialize, Debug)]
struct DataInner<T: Debug> {
    data: Option<LongArray>,
    palette: Vec<T>,
}

impl<T: Debug> BlockData<T> {
    pub fn at(&self, x: usize, sec_y: usize, z: usize) -> Option<&T> {
        let state_index = (sec_y * 16 * 16) + z * 16 + x;
        self.inner.at(state_index, 4)
    }
}

impl<T: Debug> BiomeData<T> {
    pub fn at(&self, x: usize, sec_y: usize, z: usize) -> Option<&T> {
        // Caution: int division, so lops of remainder of 4, so you can't just
        // remove a *4 and /4 and get the same results.
        let x = x / 4;
        let y = sec_y / 4;
        let z = z / 4;

        let state_index = (y * 4 * 4) + z * 4 + x;
        self.inner.at(state_index, 1)
    }
}

impl<T: Debug> DataInner<T> {
    pub fn at(&self, index: usize, min_bits_per_item: usize) -> Option<&T> {
        // TODO: Can potentially calculate this at deserialize time.
        let bits = std::cmp::max(
            (self.palette.len() as f64).log2().ceil() as usize,
            min_bits_per_item,
        );

        let values_per_64bits = 64 / bits;

        let long_index = index / values_per_64bits;
        let inter_index = index % values_per_64bits;
        let range = inter_index * bits..(inter_index + 1) * bits;

        let data = self.data.as_ref()?;
        let long = data[long_index];
        let palette_index = long.get_bits(range);

        self.palette.get(palette_index as usize)
    }
}
