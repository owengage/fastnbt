# fastnbt project
[![Latest Version]][crates.io]
[![docs-shield]][docs]
[![Build Status]][actions]

[docs]: https://docs.rs/fastnbt/latest/fastnbt/index.html
[docs-shield]: https://img.shields.io/docsrs/fastnbt

[Build Status]:
https://img.shields.io/github/workflow/status/owengage/fastnbt/Rust/master
[actions]: https://github.com/owengage/fastnbt/actions?query=branch%3Amaster
[Latest Version]: https://img.shields.io/crates/v/fastnbt.svg
[crates.io]: https://crates.io/crates/fastnbt

NBT deserializer and in-browser Rust-to-WASM powered Minecraft map renderer.

# Demo
Demo of Hermitcraft season 7 and more at [owengage.com/anvil](https://owengage.com/anvil/?world=hermitcraft7)

![alt rendered map](demo.png)  

This repository contains multiple related projects.

* [fastnbt](fastnbt/README.md): Fast (or trying to be!) serde deserializer
  for *Minecraft: Java Edition*'s NBT data format.
* fastanvil: For rendering Minecraft worlds to maps.
* fastnbt-tools: Various tools for NBT/Anvil, notably a map renderer.

Aim to support only the latest version of Minecraft. Works with 1.17 worlds and
1.16 worlds at the moment. Might work for more. Endevour to support old chunks
in worlds, but not extracting textures from older versions due to the added
complexity it would require.

The `anvil` binary from `fastnbt-tools` can render your world leveraging all of
your CPU.

See [fastnbt's README](fastnbt/README.md) for performance comparison.

# Serde deserializer example

 This example demonstrates printing out a players inventory and ender chest
 contents from the [player dat
 files](https://minecraft.gamepedia.com/Player.dat_format) found in worlds. We
 leverage serde's renaming attribute to have rustfmt conformant field names, use
 lifetimes to save on some string allocations, and use the `Value` type to
 deserialize a field we don't specify the exact structure of.

```rust
use fastnbt::error::Result;
use fastnbt::{de::from_bytes, Value};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PlayerDat<'a> {
    data_version: i32,

    #[serde(borrow)]
    inventory: Vec<InventorySlot<'a>>,
    ender_items: Vec<InventorySlot<'a>>,
}

#[derive(Deserialize, Debug)]
struct InventorySlot<'a> {
    id: &'a str,        // We avoid allocating a string here.
    tag: Option<Value>, // Also get the less structured properties of the object.

    // We need to rename fields a lot.
    #[serde(rename = "Count")]
    count: i8,
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let file = std::fs::File::open(args[0].clone()).unwrap();

    // Player dat files are compressed with GZip.
    let mut decoder = GzDecoder::new(file);
    let mut data = vec![];
    decoder.read_to_end(&mut data).unwrap();

    let player: Result<PlayerDat> = from_bytes(data.as_slice());

    println!("{:#?}", player);
}
```

# Development priorities

These are the proirities for the project when it comes to development. Ideally
we never sacrifice an earlier priority for the sake of a later one.

1. Correctness. Worlds are rendered as accurately as possible.
2. Speed. Worlds are rendered as fast as possible.
3. Memory. Worlds are rendered without sucking up RAM.

## Usage

For the libraries

```toml
[dependencies]
fastnbt = "0.18"
fastanvil = "0.18"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```