use crate::{complete, Chunk, HeightMode, JavaChunk, Region};

fn get_test_chunk() -> Vec<(JavaChunk, complete::Chunk)> {
    //todo better test region (different bioms)
    let file = std::fs::File::open("./resources/1.19.4.mca").unwrap();

    let mut region = Region::from_stream(file).unwrap();
    let data = region.read_chunk(0, 0).unwrap().unwrap();

    let current_java_chunk = JavaChunk::from_bytes(data.as_slice()).unwrap();
    let complete_chunk_current = complete::Chunk::from_bytes(data.as_slice()).unwrap();

    let chunk_pre18 =
        JavaChunk::from_bytes(include_bytes!("../../resources/1.17.1.chunk")).unwrap();

    let complete_chunk_pre18 =
        complete::Chunk::from_bytes(include_bytes!("../../resources/1.17.1.chunk")).unwrap();

    let chunk_pre13 =
        JavaChunk::from_bytes(include_bytes!("../../resources/1.12.chunk")).unwrap();

    let complete_chunk_pre13 =
        complete::Chunk::from_bytes(include_bytes!("../../resources/1.12.chunk")).unwrap();

    vec![
        (current_java_chunk, complete_chunk_current),
        (chunk_pre18, complete_chunk_pre18),
        (chunk_pre13, complete_chunk_pre13),
    ]
}

#[test]
fn block_returns_same_as_current_java_chunk() {
    let chunks = get_test_chunk();

    for (java_chunk, complete_chunk) in chunks {
        for x in 0..16 {
            for z in 0..16 {
                for y in complete_chunk.y_range() {
                    let actual = complete_chunk.block(x, y, z).unwrap().name();
                    let expected = java_chunk.block(x, y, z).unwrap().name();

                    assert!(
                        actual.eq(expected),
                        "Block at {} {}  {} was {} but should be {}",
                        x,
                        y,
                        z,
                        actual,
                        expected
                    )
                }
            }
        }
    }
}

#[test]
fn iter_block_returns_same_as_current_java_chunk() {
    let chunks = get_test_chunk();

    for (java_chunk, complete_chunk) in chunks {
        for (index, block) in complete_chunk.iter_blocks().enumerate() {
            let x = index % 16;
            let z = (index / 16) % 16;

            //- y_range() because chunk does not begin at y = 0
            let y = index as isize / (16 * 16) + complete_chunk.y_range().start;

            let actual = block.name();
            let expected = java_chunk.block(x, y, z).unwrap().name();

            assert!(
                actual.eq(expected),
                "Block at {} {} {} was {} but should be {}",
                x,
                y,
                z,
                actual,
                expected
            )
        }
    }
}

#[test]
fn biome_returns_same_as_current_java_chunk() {
    let chunks = get_test_chunk();

    for (java_chunk, complete_chunk) in chunks {
        for x in 0..16 {
            for z in 0..16 {
                for y in complete_chunk.y_range() {
                    assert_eq!(
                        complete_chunk.biome(x, y, z).unwrap(),
                        java_chunk.biome(x, y, z).unwrap(),
                    )
                }
            }
        }
    }
}

#[test]
fn surface_height_returns_same_as_current_java_chunk() {
    let chunks = get_test_chunk();

    for (java_chunk, complete_chunk) in chunks {
        for x in 0..16 {
            for z in 0..16 {
                assert_eq!(
                    complete_chunk.surface_height(x, z, HeightMode::Trust),
                    java_chunk.surface_height(x, z, HeightMode::Trust)
                )
            }
        }
    }
}
