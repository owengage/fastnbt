use fastnbt::anvil;
use fastnbt::nbt;
use flate2::read::ZlibDecoder;
use std::io;
use std::io::Read;

use fastnbt::anvil2::Chunk;
use fastnbt::nbt2::de::from_bytes;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    //let file = std::fs::File::open(args[0].clone()).unwrap();
    let file =
        std::fs::File::open("/home/ogage/minecloud/cliff-side/world/region/r.0.0.mca").unwrap();

    let mut region = anvil::Region::new(file);
    let loc = region.chunk_location(0, 0).unwrap();

    let mut chunk_data = vec![0u8; loc.sector_count * anvil::SECTOR_SIZE];
    region.load_chunk(&loc, chunk_data.as_mut_slice()).unwrap();

    let mut decoder = ZlibDecoder::new(&chunk_data[5..]);
    let mut data = vec![];
    decoder.read_to_end(&mut data).unwrap();

    let chunk: Chunk = from_bytes(data.as_slice()).unwrap();

    println!("{:?}", chunk);
}
