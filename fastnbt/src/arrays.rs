use std::ops::{Deref, DerefMut};

use byteorder::{BigEndian, ReadBytesExt};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_bytes::ByteBuf;

pub(crate) const BYTE_ARRAY_TOKEN: &str = "__fastnbt_byte_array";
pub(crate) const INT_ARRAY_TOKEN: &str = "__fastnbt_int_array";
pub(crate) const LONG_ARRAY_TOKEN: &str = "__fastnbt_long_array";

/// NBT ByteArray that owns its data. This type preserves the exact NBT type
/// when (de)serializing. This dereferences into a i8 slice, so should be usable
/// basically anywhere a slice should be.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "arbitrary1", derive(arbitrary::Arbitrary))]
pub struct ByteArray {
    data: Vec<i8>,
}

impl Serialize for ByteArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Inner {
            __fastnbt_byte_array: ByteBuf,
        }

        Inner {
            __fastnbt_byte_array: ByteBuf::from(self.to_bytes()),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ByteArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor;
        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = ByteArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("byte array")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let token = map.next_key::<&str>()?.ok_or_else(|| {
                    serde::de::Error::custom("expected NBT byte array token, but got empty map")
                })?;
                let data = map.next_value::<ByteBuf>()?;

                if token == BYTE_ARRAY_TOKEN {
                    Ok(ByteArray::from_buf(data.into_vec()))
                } else {
                    Err(serde::de::Error::custom("expected NBT byte array token"))
                }
            }
        }
        deserializer.deserialize_map(InnerVisitor)
    }
}

impl ByteArray {
    /// Create a new ByteArray from the given data.
    pub fn new(data: Vec<i8>) -> Self {
        Self { data }
    }

    /// Move the raw data out of this ByteArray.
    pub fn into_inner(self) -> Vec<i8> {
        self.data
    }

    /// Produce a ByteArray from raw data.
    pub(crate) fn from_bytes(data: &[u8]) -> Self {
        // Safe to treat [u8] as [i8].
        let data = unsafe { &*(data as *const [u8] as *const [i8]) };
        ByteArray {
            data: data.to_owned(),
        }
    }

    /// Produce a ByteArray from raw data.
    pub(crate) fn from_buf(data: Vec<u8>) -> Self {
        // TODO: Remove copy.
        let data = data.as_slice();

        // Safe to treat [u8] as [i8].
        let data = unsafe { &*(data as *const [u8] as *const [i8]) };
        ByteArray {
            data: data.to_owned(),
        }
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        self.data.iter().flat_map(|i| i.to_be_bytes()).collect()
    }
}

impl Deref for ByteArray {
    type Target = [i8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for ByteArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// NBT IntArray that owns its data. This type preserves the exact NBT type
/// when (de)serializing. This dereferences into a i32 slice, so should be usable
/// basically anywhere a slice should be.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "arbitrary1", derive(arbitrary::Arbitrary))]
pub struct IntArray {
    data: Vec<i32>,
}

impl IntArray {
    /// Create a new IntArray from the given data.
    pub fn new(data: Vec<i32>) -> Self {
        Self { data }
    }

    /// Move the raw data out of this IntArray.
    pub fn into_inner(self) -> Vec<i32> {
        self.data
    }

    /// Produce a IntArray from raw data. This data should be big endian!
    pub(crate) fn from_bytes(data: &[u8]) -> std::io::Result<Self> {
        let data = data
            .chunks_exact(4)
            .map(|mut bs| bs.read_i32::<BigEndian>())
            .collect::<std::io::Result<Vec<i32>>>()?;

        Ok(IntArray { data })
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        self.data.iter().flat_map(|i| i.to_be_bytes()).collect()
    }
}

impl Serialize for IntArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Inner {
            __fastnbt_int_array: ByteBuf,
        }

        Inner {
            __fastnbt_int_array: ByteBuf::from(self.to_bytes()),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IntArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor;
        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = IntArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("int array")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let token = map.next_key::<&str>()?.ok_or_else(|| {
                    serde::de::Error::custom("expected NBT int array token, but got empty map")
                })?;
                let data = map.next_value::<ByteBuf>()?;

                match token {
                    INT_ARRAY_TOKEN => IntArray::from_bytes(&data)
                        .map_err(|_| serde::de::Error::custom("could not read i32 for int array")),
                    _ => Err(serde::de::Error::custom("expected NBT int array token")),
                }
            }
        }
        deserializer.deserialize_map(InnerVisitor)
    }
}

impl Deref for IntArray {
    type Target = [i32];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for IntArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// NBT LongArray that owns its data. This type preserves the exact NBT type
/// when (de)serializing. This dereferences into a i64 slice, so should be usable
/// basically anywhere a slice should be.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "arbitrary1", derive(arbitrary::Arbitrary))]
pub struct LongArray {
    data: Vec<i64>,
}

impl LongArray {
    /// Create a new LongArray from the given data.
    pub fn new(data: Vec<i64>) -> Self {
        Self { data }
    }

    /// Move the raw data out of this LongArray.
    pub fn into_inner(self) -> Vec<i64> {
        self.data
    }

    pub(crate) fn from_bytes(data: &[u8]) -> std::io::Result<Self> {
        let data = data
            .chunks_exact(8)
            .map(|mut bs| bs.read_i64::<BigEndian>())
            .collect::<std::io::Result<Vec<i64>>>()?;

        Ok(LongArray { data })
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        self.data.iter().flat_map(|i| i.to_be_bytes()).collect()
    }
}

impl Serialize for LongArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Inner {
            __fastnbt_long_array: ByteBuf,
        }

        Inner {
            __fastnbt_long_array: ByteBuf::from(self.to_bytes()),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LongArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct InnerVisitor;
        impl<'de> Visitor<'de> for InnerVisitor {
            type Value = LongArray;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("long array")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let token = map.next_key::<&str>()?.ok_or_else(|| {
                    serde::de::Error::custom("expected NBT long array token, but got empty map")
                })?;
                let data = map.next_value::<ByteBuf>()?;

                match token {
                    LONG_ARRAY_TOKEN => LongArray::from_bytes(&data)
                        .map_err(|_| serde::de::Error::custom("could not read i64 for long array")),
                    _ => Err(serde::de::Error::custom("expected NBT long array token")),
                }
            }
        }
        deserializer.deserialize_map(InnerVisitor)
    }
}

impl Deref for LongArray {
    type Target = [i64];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for LongArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
