# fastsnbt crate

Documentation: [docs.rs](https://docs.rs/crate/fastsnbt)

Fast serde deserializer and serializer for *Minecraft: Java Edition*'s sNBT format.

Zero-copy is supported where possible through `from_str`.

[See fastnbt's documentation](https://docs.rs/crate/fastnbt) for more information.

```toml
[dependencies]
fastsnbt = "0.2"
```

`fastsnbt` follows Semver, some things that this project does *not* count as a
breaking change are:

* Minimum Rust version change. Outside of corporate environments this should not
  be too difficult, and I don't see much need for sNBT in those environments.
* Improving the (de)serializer such that valid sNBT that did not (de)serialize, then
  (de)serializes. Any of these cases I consider a bug.

Changes that make `fastsnbt` incompatible with WebAssembly *are* considered
breaking changes.

## NBT crate

`fastsnbt` tightly cooperates with
[`fastnbt`](https://github.com/owengage/fastnbt/blob/master/fastnbt/README.md).
It serves more as an extension to `fastnbt` than a standalone crate.
For NBT types, `Value` etc. see [fastnbt's docs here](https://docs.rs/crate/fastnbt).
