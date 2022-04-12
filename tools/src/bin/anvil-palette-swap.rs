use std::{collections::HashMap, fs::File};

use clap::{App, Arg};
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
    let matches = App::new("anvil-palette-swap")
        .arg(Arg::with_name("region").required(true))
        .arg(
            Arg::with_name("from")
                .long("from")
                .short("f")
                .takes_value(true)
                .required(true)
                .help("blockstate to transform from, eg minecraft:oak_leaves"),
        )
        .arg(
            Arg::with_name("out-file")
                .long("out-file")
                .short("o")
                .takes_value(true)
                .required(true)
                .help("full path to write the resulting region file"),
        )
        .arg(
            Arg::with_name("to")
                .long("to")
                .short("t")
                .takes_value(true)
                .required(true)
                .help("blockstate to transform to, eg minecraft:diamond_block"),
        )
        .get_matches();

    let region = matches.value_of_os("region").unwrap();
    let out_path = matches.value_of_os("out-file").unwrap();
    let from = matches.value_of("from").unwrap();
    let to = matches.value_of("to").unwrap();

    let region = File::open(region).unwrap();
    let mut region = Region::from_stream(region).unwrap();

    let out_file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(out_path)
        .unwrap();

    let mut new_region = Region::new(out_file).unwrap();

    for z in 0..32 {
        for x in 0..32 {
            match region.read_chunk(x, z) {
                Ok(data) => {
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
                Err(fastanvil::Error::ChunkNotFound) => {}
                Err(e) => eprintln!("{e}"),
            }
        }
    }
}
