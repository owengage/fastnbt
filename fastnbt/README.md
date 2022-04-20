# fastnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

Fast deserializer and serializer for *Minecraft: Java Edition*'s NBT format.

Includes a `Value` type for serializing or deserializing any NBT. `Value`
correctly preserves the exact NBT structure. The `nbt!` macro allows easy
creation of these values.

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

# Other NBT crates

There appears to be a few crates that support serde (de)serialization, the main
ones I found were:

* [`hematite_nbt`](https://github.com/PistonDevelopers/hematite_nbt)
* [`quartz_nbt`](https://github.com/Rusty-Quartz/quartz_nbt)

There are likely others! There are definitely more without serde support.

* All these crates support serialization and deserialization with
  serde.
* They are not interoperable with eachother due to requiring custom handling of
  NBT Array types.
* They all handle Minecraft's (actually Java's) specialised Unicode.
* quartz and fastnbt support borrowing from the underlying bytes being deserialized.
* fastnbt's `Value` type can round-trip deserialize-serialize NBT arrays. The
  other crates have value types as well, they may also rount-trip correctly.

  Honestly, they all seem like good options!