#[cfg(feature = "render")]
use crate::{biome::Biome, Block, Palette, Rgba};
#[cfg(feature = "render")]
use std::hash::Hash;
#[cfg(feature = "render")]
use std::{collections::hash_map::DefaultHasher, hash::Hasher};

use fastnbt::{nbt, LongArray, Value};

mod region;
mod rogue_chunks;
mod section_data;
mod complete_chunk;
#[cfg(feature = "render")]
mod standard_chunks;
mod unicode_chunk;

#[test]
fn nbt_macro_use() {
    // this checks that the fastnbt macro is accessible from an other crate.
    let val = nbt!([L;1,2,3]);
    assert_eq!(val, Value::LongArray(LongArray::new(vec![1, 2, 3])));
}

/// A palette that colours blocks based on the hash of their full description.
/// Will produce gibberish looking maps but is great for testing rendering isn't
/// changing.
#[cfg(feature = "render")]
pub struct HashPalette;

#[cfg(feature = "render")]
impl Palette for HashPalette {
    fn pick(&self, block: &Block, _: Option<Biome>) -> Rgba {
        // Call methods just to exercise all the code.
        block.name();
        block.snowy();
        let hash = calculate_hash(block.encoded_description());
        let bytes = hash.to_be_bytes();
        [bytes[0], bytes[1], bytes[2], 255]
    }
}

#[cfg(feature = "render")]
fn calculate_hash<T: Hash + ?Sized>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
