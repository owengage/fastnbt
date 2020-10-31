# fastnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

Fast (or trying to be!) deserializer and parser for *Minecraft: Java Edition*'s NBT format.

Includes

* a serde based deserializer for NBT for deserialising into `struct`s.
* a parser for low memory and/or NBT with unknown structure.

Serde deserialization is implemented in a way to try and avoid memory allocations. Strings can be deserialized as `&str`, as well as deserialising the block states of chunks to `&[u8]`.

See the `examples` directory for more examples.

```toml
[dependencies]
fastnbt = "0.10"
```

If you only need the unstructured parser and do not need to use the serde
deserializer, you can save on compile time by disabling default features.

```toml
[dependencies]
fastnbt = { version="0.10", default_features=false }
```

# Serde derserialize example

This example shows retrieving a players inventory from the palyer.dat file found
in the world/playerdata directory.

In particular we are avoiding allocating a new string for every inventory slot
by instead having a &str with a lifetime tied to the input data.

```rust
use fastnbt::de::from_bytes;
use fastnbt::error::Result;
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
    id: &'a str, // we avoid allocating a string here.
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
```
