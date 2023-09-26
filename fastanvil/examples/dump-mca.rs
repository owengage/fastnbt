use std::fs::File;

use fastanvil::Region;
use fastnbt::error::Result;
use fastnbt::Value;

//
// This loads an MCA file:
// https://minecraft.wiki/w/Anvil_file_format#Further_information
//
// This can be a region, a POI, or an entities MCA file, and dumps the value of it.
//

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let file = File::open(path).unwrap();

    let mut mca = Region::from_stream(file).unwrap();

    for chunk in mca.iter().flatten() {
        let chunk: Result<Value> = fastnbt::from_bytes(&chunk.data);
        println!("{:#?}", chunk);
    }
}
