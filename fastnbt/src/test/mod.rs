use std::convert::TryFrom;

use serde::{Deserialize, Serialize};

use crate::Tag;

#[allow(clippy::float_cmp)]
mod de;

#[allow(clippy::float_cmp)]
mod value;

pub mod builder;
mod fuzz;
mod macros;
mod minecraft_chunk;
mod resources;
mod ser;
mod stream;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Single<T: Serialize> {
    val: T,
}

#[derive(Serialize, Deserialize)]
struct Wrap<T: Serialize>(T);

macro_rules! check_tags {
    {$($tag:ident = $val:literal),* $(,)?} => {
        $(
            assert_eq!(u8::from(Tag::$tag), $val);
        )*
    };
}

#[test]
fn exhaustive_tag_check() {
    check_tags! {
        End = 0,
        Byte = 1,
        Short = 2,
        Int = 3,
        Long = 4,
        Float = 5,
        Double = 6,
        ByteArray = 7,
        String = 8,
        List = 9,
        Compound = 10,
        IntArray = 11,
        LongArray = 12,
    }

    for value in 13..=u8::MAX {
        assert!(Tag::try_from(value).is_err())
    }
}
