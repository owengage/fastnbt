# fastnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

Fast (or trying to be!) deserailizer and parser for *Minecraft: Java Edition*'s NBT and Anvil formats.

Includes

* a serde based deserializer for NBT for deserialising into `struct`s.
* a parser for low memory and/or NBT with unknown structure.
* various types for handling Anvil region files.

Serde deserialization is implemented in a way to try and avoid memory allocations. Strings can be deserialized as `&str`, as well as deserialising the block states of chunks to `&[u8]`.

See the `examples` directory for more examples.

# Serde derserialize example

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
    id: &'a str, // we avoid allocating a string here.
}
```
