use std::ops::Deref;

use serde::{Deserialize, Serialize};
use serde_bytes::Bytes;

use crate::{CompTag, BYTE_ARRAY_TAG, INT_ARRAY_TAG, LONG_ARRAY_TAG};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct ByteArray {
    tag: CompTag<BYTE_ARRAY_TAG>,
    data: Vec<i8>,
}

impl Serialize for ByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // We can't know anything about NBT here, since we might be serializing
        // to a different format. But we can create a hidden inner type to
        // signal the serializer.
        #[derive(Serialize)]
        #[allow(non_camel_case_types)]
        enum __fastnbt_byte_array<'a> {
            ByteArray(&'a Bytes),
        }

        // Safe to treat [i8] as [u8].
        let data = unsafe { &*(self.data.as_slice() as *const [i8] as *const [u8]) };
        let array = __fastnbt_byte_array::ByteArray(Bytes::new(data));

        array.serialize(serializer)
    }
}

impl ByteArray {
    pub fn new(data: Vec<i8>) -> Self {
        Self { tag: CompTag, data }
    }
}

impl Deref for ByteArray {
    type Target = Vec<i8>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct IntArray {
    tag: CompTag<INT_ARRAY_TAG>,
    data: Vec<i32>,
}

impl IntArray {
    pub fn new(data: Vec<i32>) -> Self {
        Self { tag: CompTag, data }
    }
}

impl Serialize for IntArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // We can't know anything about NBT here, since we might be serializing
        // to a different format. But we can create a hidden inner type to
        // signal the serializer.
        #[derive(Serialize)]
        #[allow(non_camel_case_types)]
        enum __fastnbt_int_array<'a> {
            Data(&'a Bytes),
        }

        // Alignment of i32 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.as_slice().align_to::<u8>() };

        let array = __fastnbt_int_array::Data(Bytes::new(data));

        array.serialize(serializer)
    }
}

impl Deref for IntArray {
    type Target = Vec<i32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
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

impl Serialize for LongArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // We can't know anything about NBT here, since we might be serializing
        // to a different format. But we can create a hidden inner type to
        // signal the serializer.
        #[derive(Serialize)]
        #[allow(non_camel_case_types)]
        enum __fastnbt_long_array<'a> {
            Data(&'a Bytes),
        }

        // Alignment of i64 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.as_slice().align_to::<u8>() };

        let array = __fastnbt_long_array::Data(Bytes::new(data));

        array.serialize(serializer)
    }
}

impl Deref for LongArray {
    type Target = Vec<i64>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
