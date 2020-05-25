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
/// `Parser` has some usage examples which might be helpful.
pub mod nbt;
