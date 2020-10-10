//! Aims to allow fast parsing of NBT and Anvil data from *Minecraft: Java Edition*.
//!
//! A `serde` compatible deserializer can be found in the `nbt` module. This deserialiser works on
//! an in-memory `&[u8]`, meaning you need all of the NBT data in memory. This has the advantage of
//! allowing you to avoid memory allocations in some cases. See the `de` module for more information.
//!
//! If you require accessing large amount of NBT data that you do not want to keep in memory, you can use
//! the `stream` module. This does not allow you to deserialize into Rust `struct`s, but does allow
//! low memory footprint processing on NBT data.
//!
//! `stream` is also useful when you do not know the structure ahead of time.
//!
//! Both this crate and the tools crate are under one [fastnbt Github repository](https://github.com/owengage/fastnbt)

pub mod anvil;
pub mod tex;

mod nbt;

pub use nbt::de;
pub use nbt::error;
pub use nbt::stream;
