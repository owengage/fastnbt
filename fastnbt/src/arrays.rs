use std::ops::Deref;

use serde::{de::Error, Deserialize};

const BYTE_ARRAY_TAG: u8 = 7;
const INT_ARRAY_TAG: u8 = 11;
const LONG_ARRAY_TAG: u8 = 12;

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

#[derive(Debug, Clone, Copy)]
struct CompTag<const N: u8>;

impl<'de, const N: u8> Deserialize<'de> for CompTag<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tag: u8 = Deserialize::deserialize(deserializer)?;
        if tag != N {
            Err(Error::custom("unexpected array type"))
        } else {
            Ok(Self)
        }
    }
}
