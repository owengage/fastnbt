use fastanvil::{complete, CurrentJavaChunk, Region};
use fastnbt::from_bytes;

fn main() {
    let file = std::fs::File::open("./fastanvil/resources/1.18.mca").unwrap();

    let mut region = Region::from_stream(file).unwrap();
    let data = region.read_chunk(0, 0).unwrap().unwrap();

    let java_chunk: CurrentJavaChunk = from_bytes(data.as_slice()).unwrap();

    let complete_chunk = complete::chunk::Chunk::from_current_chunk(&java_chunk);

    println!("{}", complete_chunk.status);
    println!("{}", java_chunk.status);
}
