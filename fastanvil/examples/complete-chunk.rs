use fastanvil::{complete, Chunk, Region};

fn main() {
    let file = std::fs::File::open("./fastanvil/resources/1.19.4.mca").unwrap();

    let mut region = Region::from_stream(file).unwrap();

    let data = region.read_chunk(0, 0).unwrap().unwrap();

    let complete_chunk = complete::Chunk::from_bytes(&data).unwrap();

    println!("{}", complete_chunk.status);

    println!("{:?}", complete_chunk.y_range());
}
