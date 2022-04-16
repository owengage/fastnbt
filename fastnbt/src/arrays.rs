use std::ops::Deref;

use byteorder::{BigEndian, ReadBytesExt};
use serde::{de::Visitor, Deserialize, Serialize};
use serde_bytes::Bytes;

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
                let data = map.next_value::<&[u8]>()?;

                if token == BYTE_ARRAY_TOKEN {
                    Ok(ByteArray::from_bytes(data))
                } else {
                    Err(serde::de::Error::custom("expected NBT byte array token"))
                }
            }
        }
        deserializer.deserialize_newtype_struct(BYTE_ARRAY_TOKEN, InnerVisitor)
    }
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
        enum Inner<'a> {
            __fastnbt_byte_array(&'a Bytes),
        }

        // Safe to treat [i8] as [u8].
        let data = unsafe { &*(self.data.as_slice() as *const [i8] as *const [u8]) };
        let array = Inner::__fastnbt_byte_array(Bytes::new(data));

        array.serialize(serializer)
    }
}

impl ByteArray {
    /// Create a new ByteArray from the given data.
    pub fn new(data: Vec<i8>) -> Self {
        Self { data }
    }

    /// Produce a ByteArray from raw data.
    pub(crate) fn from_bytes(data: &[u8]) -> Self {
        // Safe to treat [u8] as [i8].
        let data = unsafe { &*(data as *const [u8] as *const [i8]) };
        ByteArray {
            data: data.to_owned(),
        }
    }
}

impl Deref for ByteArray {
    type Target = [i8];

    fn deref(&self) -> &Self::Target {
        &self.data
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

    /// Produce a IntArray from raw data.
    pub(crate) fn from_bytes(data: &[u8]) -> std::io::Result<Self> {
        let data = data
            .chunks_exact(4)
            .map(|mut bs| bs.read_i32::<BigEndian>())
            .collect::<std::io::Result<Vec<i32>>>()?;

        Ok(IntArray { data })
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
                let data = map.next_value::<&[u8]>()?;

                if token == INT_ARRAY_TOKEN {
                    IntArray::from_bytes(data)
                        .map_err(|_| serde::de::Error::custom("could not read i32 for int array"))
                } else {
                    Err(serde::de::Error::custom("expected NBT int array token"))
                }
            }
        }
        deserializer.deserialize_newtype_struct(INT_ARRAY_TOKEN, InnerVisitor)
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
        enum Inner<'a> {
            __fastnbt_int_array(&'a Bytes),
        }

        // Alignment of i32 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.as_slice().align_to::<u8>() };

        let array = Inner::__fastnbt_int_array(Bytes::new(data));

        array.serialize(serializer)
    }
}

impl Deref for IntArray {
    type Target = [i32];

    fn deref(&self) -> &Self::Target {
        &self.data
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

    pub(crate) fn from_bytes(data: &[u8]) -> std::io::Result<Self> {
        let data = data
            .chunks_exact(8)
            .map(|mut bs| bs.read_i64::<BigEndian>())
            .collect::<std::io::Result<Vec<i64>>>()?;

        Ok(LongArray { data })
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
                let data = map.next_value::<&[u8]>()?;

                if token == LONG_ARRAY_TOKEN {
                    LongArray::from_bytes(data)
                        .map_err(|_| serde::de::Error::custom("could not read i64 for long array"))
                } else {
                    Err(serde::de::Error::custom("expected NBT long array token"))
                }
            }
        }
        deserializer.deserialize_newtype_struct(LONG_ARRAY_TOKEN, InnerVisitor)
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
        enum Inner<'a> {
            __fastnbt_long_array(&'a Bytes),
        }

        // Alignment of i64 is >= alignment of bytes so this should always work.
        let (_, data, _) = unsafe { self.data.as_slice().align_to::<u8>() };

        let array = Inner::__fastnbt_long_array(Bytes::new(data));

        array.serialize(serializer)
    }
}

impl Deref for LongArray {
    type Target = [i64];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
