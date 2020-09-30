//! Aims to allow fast parsing of NBT and Anvil data from Minecraft: Java Edition.
//!
//! Some examples can be found in `nbt::Parser`'s documentation. The `fastnbt-tools` crate contains
//! executables based on this crate which might serve as more complex examples.
//!
//! Both this crate and the tools crate are under one [fastnbt Github repository](https://github.com/owengage/fastnbt)

/// For handling Minecraft's region format, Anvil.
///
/// `anvil::Region` can be given a `Read` and `Seek` type eg a file in order to extract chunk data.
pub mod anvil;

/// For handling NBT data, which Minecraft uses for most data storage.
///
/// `de` contains a standard Serde deserializer to let you deserialize NBT into structs.
///
/// `stream` contains a parser to let you manually parse NBT, rather than putting it into a `struct`.
/// This can let you for example simply dump a bunch of NBT without knowing the size or structure.
pub mod nbt;
