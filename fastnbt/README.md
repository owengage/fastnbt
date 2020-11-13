# fastnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

Fast (or trying to be!) deserializer and parser for *Minecraft: Java Edition*'s NBT format.

Includes

* a serde based deserializer for NBT for deserialization.
* a lower level parser using the `Read` trait.

The derserializer allows you to avoid allocations where possible. Strings can be
deserialized to `&'a str` where `'a` is the lifetime of the data being
deserialized. This can also be done for lists of any integral type into 
`&'a [u8]`. The same applies to the Array types in NBT.

You can then parse the `&[u8]` only when you need it. The `fastanvil` crate has
a `PackedBits` type that can do this for you.

`fastnbt::Value` can be used to deserialize any NBT value.

See the `examples` directory for more examples.

```toml
[dependencies]
fastnbt = "0.11"
```

# Serde derserialize example

This example shows retrieving a players inventory from the palyer.dat file found
in the world/playerdata directory.

Some points:

* we are avoiding allocating a new string for every inventory slot by instead
  having a &str with a lifetime tied to the input data.
* we are deserializing the tag structure which is very dynamic using `Value`.
* we rename fields using `serde`. NBT structures tend to mix case a lot.

```rust
use fastnbt::error::Result;
use fastnbt::{de::from_bytes, Value};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::Read;

// This example show retrieving a players inventory from the palyer.dat file
// found in the world/playerdata directory.
//
// In particular we are avoiding allocating a new string for every inventory
// slot by instead having a &str with a lifetime tied to the input data.

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

# Comparison to other NBT crates

There are other crates for NBT out there, this tries to give an honest comparison to them.

| Feature | `fastnbt` | Hematite `nbt` | note |
| ------- | --------- | -------------- | ---- |
| Benchmark world render\* | 23.8+-0.7s | 31.6+-0.9s | making `fastnbt` 33% faster in this test |
| Deserialization | yes | yes | |
| Serialization | no | yes | |
| `Value`-like type | yes | yes | |
| Long Array (MC 1.12+) | yes | yes | | 
| Minecraft specialized unicode | no\*\* | yes | |
| Deserialize from reader | no | yes | |
| WASM compatible | yes | unknown | | 


\*see 01-11-2020.md in benchmarks directory

\*\*intended feature

