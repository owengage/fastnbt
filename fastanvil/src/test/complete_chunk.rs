use std::fs;
use fastnbt::from_bytes;

use crate::{complete, Chunk, CurrentJavaChunk, Region};

fn get_test_chunk() -> CurrentJavaChunk {
    let file = std::fs::File::open("./resources/1.18.mca").unwrap();

    let mut region = Region::from_stream(file).unwrap();
    let data = region.read_chunk(0, 0).unwrap().unwrap();

    let chunk: CurrentJavaChunk = from_bytes(data.as_slice()).unwrap();

    chunk
}

#[test]
fn block_returns_same_as_current_java_chunk() {
    let java_chunk = get_test_chunk();

    let complete_chunk: complete::Chunk = (&java_chunk).into();

    for x in 0..16 {
        for z in 0..16 {
            for y in -64..64 {
            // for y in complete_chunk.y_range() {
                assert!(complete_chunk
                    .block(x, y, z)
                    .unwrap()
                    .name()
                    .eq(java_chunk.block(x, y, z).unwrap().name()))
            }
        }
    }
}
