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

`fastnbt` is a serde serializer and deserializer for *Minecraft: Java Edition's*
NBT format.

This repository also contains fastanvil for rendering maps of Minecraft worlds.
An in-browser Rust-to-WASM powered Minecraft map renderer demo is below.
Supports 1.18 down to 1.15 inclusive. Eventually want to support at least down
to 1.12.

# Demo

Demo of Hermitcraft season 8 and more at [owengage.com/anvil](https://owengage.com/anvil/?world=hermitcraft8)

![alt rendered map](demo.png)  

This repository contains multiple related projects.

* [fastnbt](fastnbt/README.md): Fast serde serializer/deserializer for *Minecraft: Java
  Edition*'s NBT data format.
* fastanvil: For rendering Minecraft worlds to maps.
* fastnbt-tools: Various tools for NBT/Anvil, notably a map renderer.

The `anvil` binary from `fastnbt-tools` can render your world leveraging all of
your CPU. See [fastnbt's README](fastnbt/README.md) for performance comparison.

# Example: editing level.dat

The following edits the world spawn to 250, 200, 250 (probably not a good
idea!). Full example in fastnbt/examples directory.

```rust
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
```

# Example: print player inventory

 This example demonstrates printing out a players inventory and ender chest contents from the [player dat
 files](https://minecraft.gamepedia.com/Player.dat_format) found in worlds. We
 * use serde's renaming attribute to have rustfmt conformant field names,
 * use lifetimes to save on string allocations, and 
 * use the `Value` type to deserialize a field we don't specify the exact
   structure of.

```rust
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

## Usage

For the libraries

```toml
[dependencies]
fastnbt = "2.0.0-alpha"
fastanvil = "0.24"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```
