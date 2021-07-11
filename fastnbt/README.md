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

`fastnbt` follows Semver, so any version 1 code you write should remain valid.
Some things that this project does *not* count as a breaking change are:

* Minimum Rust version change. Outside of corporate environments this should not
  be too difficult, and I don't see much need for NBT in those environments.
* Improving the deserializer such that valid NBT that did not deserialize, then
  deserializes. Any of these cases I consider a bug.

Changes that make `fastnbt` incompatible with WebAssembly *are* considered
breaking changes.

# Comparison to other NBT crates

There are other crates for NBT out there, this tries to give an honest
comparison to them. The Hemtite `nbt` crate was the only other crate I found with serde deserialization.

| Feature | `fastnbt` | Hematite `nbt` | note |
| ------- | --------- | -------------- | ---- |
| Benchmark world render time (relative)\* | 1.00 | 1.37 | fastnbt is ~37% faster. See note. |
| Deserialization | yes | yes | |
| Serialization | no | yes | |
| `Value`-like type | yes | yes | `fastnbt` is careful to preserve exact types. |
| Long Array (MC 1.12+) | yes | yes | | 
| Minecraft specialized unicode | yes | yes | |
| Deserialize from reader | no | yes | |
| WASM compatible | yes | unknown | | 


\* This is rendering the overworld of Etho's Lets Play Episode 550. Exact relative
figures are 1000±13 for fastnbt and 1370±7 for hematite-nbt. This used the
`anvil tiles` executable, swapping out the deserializer only, so performance
tweaks in rendering chunks are not counted.
