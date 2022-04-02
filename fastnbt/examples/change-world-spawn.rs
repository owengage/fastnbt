//! This executable takes a path to a level.dat file for a world, and spits out
//! a new level.dat file in the current directory. The data is changed so that
//! the world spawn is set to 0,0.

use fastnbt::Value;
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Write},
};

#[derive(Serialize, Deserialize)]
struct LevelDat {
    #[serde(rename = "Data")]
    data: Data,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Data {
    spawn_x: i32,
    spawn_y: i32,
    spawn_z: i32,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    let file = std::fs::File::open(&args[1]).unwrap();
    let mut decoder = GzDecoder::new(file);
    let mut bytes = vec![];
    decoder.read_to_end(&mut bytes).unwrap();

    let mut leveldat: LevelDat = fastnbt::from_bytes(&bytes).unwrap();

    leveldat.data.spawn_x = 250;
    leveldat.data.spawn_y = 200;
    leveldat.data.spawn_z = 250;

    let new_bytes = fastnbt::to_bytes(&leveldat).unwrap();
    let outfile = std::fs::File::create("level.dat").unwrap();
    let mut encoder = GzEncoder::new(outfile, Compression::fast());
    encoder.write_all(&new_bytes).unwrap();
}
