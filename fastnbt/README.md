# fastnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

Fast deserializer and parser for *Minecraft: Java Edition*'s NBT format.

Includes

* a serde based deserializer for NBT for deserialization.
* a lower level parser using the `Read` trait.

The derserializer allows you to avoid allocations where possible. Strings can be
deserialized to `&'a str` where `'a` is the lifetime of the data being
deserialized. The `borrow` module contains more types for avoiding allocations.

[See the documentation](https://docs.rs/crate/fastnbt) for more information.

```toml
[dependencies]
fastnbt = "1"
```

# Comparison to other NBT crates

There are other crates for NBT out there, this tries to give an honest
comparison to them. The Hemtite `nbt` crate was the only other crate I found with serde deserialization.

| Feature | `fastnbt` | Hematite `nbt` | note |
| ------- | --------- | -------------- | ---- |
| Benchmark world render time (relative)\* | 1.00 | 1.37 | fastnbt is ~37% faster. See note. |
| Deserialization | yes | yes | |
| Serialization | no | yes | |
| `Value`-like type | yes | yes | |
| Long Array (MC 1.12+) | yes | yes | | 
| Minecraft specialized unicode | no\*\* | yes | |
| Deserialize from reader | no | yes | |
| WASM compatible | yes | unknown | | 


\* This is rendering the overworld of Etho's Lets Play Episode 550. Exact relative
figures are 1000±13 for fastnbt and 1370±7 for hematite-nbt. This used the
`anvil tiles` executable, swapping out the deserializer only, so performance
tweaks in rendering chunks are not counted.

\*\*intended feature
