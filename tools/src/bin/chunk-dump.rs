use fastnbt::anvil;
use flate2::read::ZlibDecoder;
use std::io::Read;

use fastnbt::anvil::types::Chunk;
use fastnbt::nbt2::de::from_bytes;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();

    let mut region = anvil::Region::new(file);
    let loc = region.chunk_location(0, 0).unwrap();

    let mut chunk_data = vec![0u8; loc.sector_count * anvil::SECTOR_SIZE];
    region.load_chunk(&loc, &mut chunk_data).unwrap();

    let mut decoder = ZlibDecoder::new(&chunk_data[5..]);
    let mut data = vec![];
    decoder.read_to_end(&mut data).unwrap();

    let chunk: Chunk = from_bytes(data.as_slice()).unwrap();

    println!("{:?}", chunk);
}
