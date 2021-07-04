use byteorder::{BigEndian, ReadBytesExt};
use serde::Deserialize;

use crate::{CompTag, INT_ARRAY_TAG, LONG_ARRAY_TAG};

use super::BYTE_ARRAY_TAG;

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct ByteArray<'a> {
    tag: CompTag<BYTE_ARRAY_TAG>,
    data: &'a [u8],
}

impl<'a> ByteArray<'a> {
    pub fn iter(&self) -> ByteIter<'a> {
        ByteIter(*self)
    }
}

pub struct ByteIter<'a>(ByteArray<'a>);

impl<'a> Iterator for ByteIter<'a> {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i8().ok()
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct IntArray<'a> {
    tag: CompTag<INT_ARRAY_TAG>,
    data: &'a [u8],
}

impl<'a> IntArray<'a> {
    pub fn iter(&self) -> IntIter<'a> {
        IntIter(*self)
    }
}

pub struct IntIter<'a>(IntArray<'a>);

impl<'a> Iterator for IntIter<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i32::<BigEndian>().ok()
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct LongArray<'a> {
    tag: CompTag<LONG_ARRAY_TAG>,
    data: &'a [u8],
}

impl<'a> LongArray<'a> {
    pub fn iter(&self) -> LongIter<'a> {
        LongIter(*self)
    }
}

pub struct LongIter<'a>(LongArray<'a>);

impl<'a> Iterator for LongIter<'a> {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.data.read_i64::<BigEndian>().ok()
    }
}
