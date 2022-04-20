use fastnbt::{nbt, LongArray, Value};

mod region;
mod rogue_chunks;
mod section_data;
mod standard_chunks;
mod unicode_chunk;

#[test]
fn nbt_macro_use() {
    // this checks that the fastnbt macro is accessible from an other crate.
    let val = nbt!([L;1,2,3]);
    assert_eq!(val, Value::LongArray(LongArray::new(vec![1, 2, 3])));
}
