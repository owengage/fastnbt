//! This executable takes a path to a level.dat file for a world, and spits out
//! a new level.dat file in the current directory. The data is changed so that
//! the world spawn is set to 0,0.
//!
//! This uses fastnbt::Value which isn't easy to use due to it's dynamic nature.
//! The change-world-spawn example uses an actual LevelDat struct.

use fastnbt::Value;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use std::io::{Read, Write};

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let file = std::fs::File::open(&args[1]).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut bytes = vec![];
    decoder.read_to_end(&mut bytes).unwrap();

    let mut leveldat: Value = fastnbt::from_bytes(&bytes).unwrap();

    match &mut leveldat {
        Value::Compound(level) => {
            let data = level.get_mut("Data").unwrap();
            match data {
                Value::Compound(data) => {
                    *data.get_mut("SpawnX").unwrap() = Value::Int(0);
                    *data.get_mut("SpawnY").unwrap() = Value::Int(100);
                    *data.get_mut("SpawnZ").unwrap() = Value::Int(0);
                }
                _ => panic!(),
            }
        }
        _ => panic!(),
    }

    let new_bytes = fastnbt::to_bytes(&leveldat).unwrap();
    let outfile = std::fs::File::create("level.dat").unwrap();
    let mut encoder = GzEncoder::new(outfile, Compression::fast());
    encoder.write_all(&new_bytes).unwrap();
}
