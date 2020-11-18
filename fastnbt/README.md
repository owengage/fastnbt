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

[See the documentation](https://docs.rs/crate/fastnbt) for more information.

```toml
[dependencies]
fastnbt = "0.14"
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

