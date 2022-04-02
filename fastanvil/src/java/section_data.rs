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

impl<T: Debug> BlockData<T> {
    /// Get the block data for the block at x,y,z, where x,y,z are relative to
    /// the section ie 0..16.
    pub fn at(&self, x: usize, sec_y: usize, z: usize) -> Option<&T> {
        let state_index = (sec_y * 16 * 16) + z * 16 + x;
        self.inner.at(
            state_index,
            blockstates_bits_per_block(self.inner.palette.len()),
        )
    }

    /// Get iterator for the state indicies. This will increase in x, then z,
    /// then y. These indicies are used with the relevant palette to get the
    /// data for that block.
    ///
    /// This returns None if no block states were present. This typically means
    /// the section was empty (ie filled with air).
    ///
    /// You can recover the coordinate be enumerating the iterator:
    ///
    /// ```no_run
    /// # use fastanvil::BlockData;
    /// # let states: BlockData<usize> = todo!();
    /// for (i, block_index) in states.try_iter_indices().unwrap().enumerate() {
    ///     let x = i & 0x000F;
    ///     let y = (i & 0x0F00) >> 8;
    ///     let z = (i & 0x00F0) >> 4;
    /// }
    /// ```
    pub fn try_iter_indices(&self) -> Option<StatesIter> {
        if let Some(data) = &self.inner.data {
            let bits = blockstates_bits_per_block(self.inner.palette.len());
            Some(StatesIter::new(bits, 16 * 16 * 16, data))
        } else {
            None
        }
    }

    /// Get the palette for this block data. Indicies into this palette can be
    /// obtained via [`try_iter_indices`][`BlockData::try_iter_indices`].
    pub fn palette(&self) -> &[T] {
        self.inner.palette.as_slice()
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
        self.inner
            .at(state_index, biomes_bits_per_block(self.inner.palette.len()))
    }

    pub fn try_iter_indices(&self) -> Option<StatesIter> {
        if let Some(data) = &self.inner.data {
            let bits = biomes_bits_per_block(self.inner.palette.len());
            Some(StatesIter::new(bits, 4 * 4 * 4, data))
        } else {
            None
        }
    }

    pub fn palette(&self) -> &[T] {
        self.inner.palette.as_slice()
    }
}

#[derive(Deserialize, Debug)]
struct DataInner<T: Debug> {
    data: Option<LongArray>,
    palette: Vec<T>,
}

impl<T: Debug> DataInner<T> {
    pub fn at(&self, index: usize, bits_per_item: usize) -> Option<&T> {
        if self.data.is_none() && self.palette.len() == 1 {
            return self.palette.get(0);
        }

        let data = self.data.as_ref()?;

        let values_per_64bits = 64 / bits_per_item;

        let long_index = index / values_per_64bits;
        let inter_index = index % values_per_64bits;
        let range = inter_index * bits_per_item..(inter_index + 1) * bits_per_item;

        // Super important line: treat the i64 as an u64.
        // Bug 1: Kept i64 and the get_bits interprets as signed.
        // Bug 2: Went to usize, worked on 64bit platforms broke on 32 bit like WASM.
        let long = data[long_index] as u64;

        let palette_index = long.get_bits(range);

        self.palette.get(palette_index as usize)
    }
}

// Block states at the least can be missing from the world data. This typically
// just means that it's a big block of air. We default the DataInner and let the
// fact data is None to also return none. Rather than have BlockData be optional
// in the chunk struct.
impl<T: Debug> Default for DataInner<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
            palette: Default::default(),
        }
    }
}

impl<T: Debug> Default for BlockData<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T: Debug> Default for BiomeData<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

/// Number of bits that will be used per block in block_states data for blocks.
fn blockstates_bits_per_block(palette_len: usize) -> usize {
    std::cmp::max((palette_len as f64).log2().ceil() as usize, 4)
}

/// Number of bits that will be used per block in block_states data for biomes.
fn biomes_bits_per_block(palette_len: usize) -> usize {
    std::cmp::max((palette_len as f64).log2().ceil() as usize, 1)
}

/// Iterator over block state data. Each value is the index into the relevant palette.
pub struct StatesIter<'a> {
    inner: &'a [i64], // remaining data to iterate.
    stride: usize,    // stride in bits we're iterating.
    pos: usize,       // bit position the next value starts at.
    // need to track the 'remaining' because some of the last long might be padding.
    remaining: usize,
}

impl<'a> StatesIter<'a> {
    pub(crate) fn new(stride: usize, len: usize, inner: &'a [i64]) -> Self {
        Self {
            inner,
            stride,
            remaining: len,
            pos: 0,
        }
    }
}

impl<'a> Iterator for StatesIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let start = self.pos;
        self.pos += self.stride;
        let end = self.pos;
        let datum = *(self.inner.get(0)?) as u64;

        let value = datum.get_bits(start..end) as usize;
        self.remaining -= 1;

        if self.pos + self.stride > 64 {
            self.pos = 0;
            self.inner = &self.inner[1..];
        }

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'a> ExactSizeIterator for StatesIter<'a> {}
