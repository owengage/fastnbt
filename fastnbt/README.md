# fastnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

Fast deserializer and serializer for *Minecraft: Java Edition*'s NBT format.

Includes a `Value` type for serializing or deserializing any NBT. `Value`
correctly preserves the exact NBT structure.

The derserializer allows you to avoid allocations where possible. Strings can be
deserialized to `&'a str` where `'a` is the lifetime of the data being
deserialized. The `borrow` module contains more types for avoiding allocations.

[See the documentation](https://docs.rs/crate/fastnbt) for more information.

```toml
[dependencies]
fastnbt = "2"
```

`fastnbt` follows Semver, some things that this project does *not* count as a
breaking change are:

* Minimum Rust version change. Outside of corporate environments this should not
  be too difficult, and I don't see much need for NBT in those environments.
* Improving the (de)serializer such that valid NBT that did not (de)serialize, then
  (de)serializes. Any of these cases I consider a bug.
* Data format when serializing types from fastnbt/fastanvil to other formats.
  Types in fastnbt implement `serde::Serialise` to enable spitting out to other
  data formats, but may change structure in future.

Changes that make `fastnbt` incompatible with WebAssembly *are* considered
breaking changes.

# Comparison to other NBT crates

There are other crates for NBT out there, this tries to give an honest
comparison to them. 

The Hematite `nbt` crate was the only other crate I found with serde
deserialization. 

* Both crates support serialization and deserialization with
  serde.
* They are not interoperable due to requiring custom handling of NBT Array
  types.
* They both handle Minecraft's (actually Java's) specialised Unicode.
* fastnbt supports borrowing from the underlying bytes being deserialized.
* fastnbt's `Value` type can round-trip deserialize-serialize NBT arrays.