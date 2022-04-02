#![allow(dead_code)]
use fastnbt::error::Result;
use fastnbt::from_bytes;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::Read;

// This example show retrieving a players inventory from the palyer.dat file
// found in the world/playerdata directory.

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PlayerDat {
    data_version: i32,
    inventory: Vec<InventorySlot>,
    ender_items: Vec<InventorySlot>,
}

#[derive(Deserialize, Debug)]
struct InventorySlot {
    id: String,
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();

    // Player dat files are compressed with GZip.
    let mut decoder = GzDecoder::new(file);
    let mut data = vec![];
    decoder.read_to_end(&mut data).unwrap();

    let player: Result<PlayerDat> = from_bytes(data.as_slice());

    println!("{:?}", player);
}
