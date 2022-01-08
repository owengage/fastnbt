use std::convert::{TryFrom, TryInto};

use crate::Tag;

#[allow(clippy::float_cmp)]
mod de;

#[allow(clippy::float_cmp)]
mod value;

mod builder;
mod de_arrays;
mod fuzz;
mod minecraft_chunk;
mod resources;
mod ser;
mod stream;

fn assert_try_into(tag: Tag) {
    assert_eq!(tag, (tag as u8).try_into().unwrap());
}

#[test]
fn exhaustive_tag_check() {
    use Tag::*;
    assert_try_into(End);
    assert_try_into(Byte);
    assert_try_into(Short);
    assert_try_into(Int);
    assert_try_into(Long);
    assert_try_into(Float);
    assert_try_into(Double);
    assert_try_into(ByteArray);
    assert_try_into(String);
    assert_try_into(List);
    assert_try_into(Compound);
    assert_try_into(Compound);
    assert_try_into(IntArray);
    assert_try_into(LongArray);

    for value in 13..=u8::MAX {
        assert!(Tag::try_from(value).is_err())
    }
}
