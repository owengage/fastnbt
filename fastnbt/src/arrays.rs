use std::ops::Deref;

use serde::Deserialize;

use crate::{CompTag, BYTE_ARRAY_TAG, INT_ARRAY_TAG, LONG_ARRAY_TAG};

#[derive(Deserialize, Debug, Clone)]
pub struct ByteArray {
    tag: CompTag<BYTE_ARRAY_TAG>,
    data: Vec<i8>,
}

impl Deref for ByteArray {
    type Target = Vec<i8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct IntArray {
    tag: CompTag<INT_ARRAY_TAG>,
    data: Vec<i32>,
}

impl Deref for IntArray {
    type Target = Vec<i32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct LongArray {
    tag: CompTag<LONG_ARRAY_TAG>,
    data: Vec<i64>,
}

impl LongArray {
    pub fn new(data: Vec<i64>) -> Self {
        Self {
            tag: CompTag::<LONG_ARRAY_TAG>,
            data,
        }
    }
}

impl Deref for LongArray {
    type Target = Vec<i64>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
