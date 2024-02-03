//! Take a region and produce a copy where a given palette item has been
//! changed to something else.
//!
//! This can for example change all oak_leaves into diamond_blocks.
//!
use std::{collections::HashMap, env, fs::File};

use fastanvil::Region;
use fastnbt::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Chunk {
    sections: Vec<Section>,

    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Section {
    block_states: Blockstates,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct Blockstates {
    palette: Vec<PaletteItem>,
    #[serde(flatten)]
    other: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize)]
struct PaletteItem {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Properties")]
    properties: Option<Value>,
}

fn main() {
    let region = env::var_os("REGION").unwrap();
    let out_path = env::var_os("OUT_FILE").unwrap();
    let from = env::var("FROM").unwrap();
    let to = env::var("TO").unwrap();

    let region = File::open(region).unwrap();
    let mut region = Region::from_stream(region).unwrap();

    // File needs to readable as well.
    let out_file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(out_path)
        .unwrap();

    let mut new_region = Region::create(out_file).unwrap();

    for z in 0..32 {
        for x in 0..32 {
            match region.read_chunk(x, z) {
                Ok(Some(data)) => {
                    let mut chunk: Chunk = fastnbt::from_bytes(&data).unwrap();
                    for section in chunk.sections.iter_mut() {
                        let palette: &mut Vec<PaletteItem> = &mut section.block_states.palette;
                        for item in palette {
                            if item.name == from {
                                item.name = to.to_owned();
                            }
                        }
                    }
                    let ser = fastnbt::to_bytes(&chunk).unwrap();
                    new_region.write_chunk(x, z, &ser).unwrap();
                }
                Ok(None) => {}
                Err(e) => eprintln!("{e}"),
            }
        }
    }
}
