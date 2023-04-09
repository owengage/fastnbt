use std::time::Instant;

use fastanvil::chunk_block_iter::{ChunkBlockIter, CurrentChunkBlockIter};
use fastanvil::{Chunk, JavaChunk, Region};

fn main() {
    let file = std::fs::File::open("./fastanvil/resources/r.0.0.mca").unwrap();

    let mut region = Region::from_stream(file).unwrap();
    let data = region.read_chunk(0, 0).unwrap().unwrap();

    let mut chunk = JavaChunk::from_bytes(&data).unwrap();

    let now = Instant::now();
    let mut count = 0;

    for _ in ChunkBlockIter::new(&mut chunk) {
        count += 1;
    }

    println!("basic {:?}", now.elapsed());

    match chunk {
        JavaChunk::Post18(mut chunk) => {

            let now = Instant::now();
            let mut count = 0;

            for x in 0..16 {
                for z in 0..16 {
                    for y in -64..320 {
                        if chunk.block(x, y, z).is_some() {
                            count += 1;
                        }
                    }
                }
            }

            println!("hand {:?}", now.elapsed());

            let now = Instant::now();
            let mut count = 0;

            for _ in CurrentChunkBlockIter::new(&mut chunk) {
                count += 1;
            }

            println!("complex {:?}", now.elapsed());
        }
        JavaChunk::Pre18(_) => {}
        JavaChunk::Pre13(_) => {}
    }
}
