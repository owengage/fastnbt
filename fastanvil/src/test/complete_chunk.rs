use fastnbt::from_bytes;

use crate::{complete, Chunk, CurrentJavaChunk, Region};

fn get_test_chunk() -> CurrentJavaChunk {
    let file = std::fs::File::open("./resources/1.19.4.mca").unwrap();

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
            for y in complete_chunk.y_range() {
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

#[test]
fn iter_block_returns_same_as_current_java_chunk() {
    let java_chunk = get_test_chunk();

    let complete_chunk: complete::Chunk = (&java_chunk).into();

    for (index, block) in complete_chunk.iter_blocks().enumerate() {
        let x = index % 16;
        let z = (index / 16) % 16;

        //- y_range() because chunk does not begin at y = 0
        let y = index as isize / (16 * 16) + complete_chunk.y_range().start;

        assert!(block.name().eq(java_chunk.block(x, y, z).unwrap().name()))
    }
}
