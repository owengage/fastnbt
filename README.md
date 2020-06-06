# fastnbt

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

A fast (or trying to be!) parser for *Minecraft: Java Edition*'s NBT and Anvil formats.

The `anvil` binary can render your world leveraging all of your CPU. On a Ryzen 3600X 6-core, with a reasonably complex world, it can render a map of 256 *regions* in under 10 seconds. That's about 30k chunks every second.

```bash
cargo install fastnbt-tools
anvil-palette ~/.minecraft/versions/1.15.2/unpacked # this is to an unzipped minecraft version JAR
anvil render ~/path/to/world-dir --palette=palette.tar # render entire overworld
anvil render ~/path/to/world-dir --dimension=end --palette=palette.tar # render entire end
anvil render ~/path/to/world-dir --size=6,6  --palette=palette.tar # render 6 by 6 regions around 0,0.
anvil render ~/path/to/world-dir --size=10,10 --offset=-4,10  --palette=palette.tar # render 10 by 10 offset by x: -4, z: 10.
```

![alt rendered map](map.png)

## TODO

* Fix end dimension panic.
* Full palette for block-based world rendering (unknown blocks are magenta at the moment)
* Modify palette colour based on height.
* Change to visitor-based parser to avoid allocation of Array tags when not needed.
* Test on Windows.
* Use newtypes idiom for the various co-ordinate types for safety.
* serde for deserialisation.
* Maybe: some sort of interactive map. WASM?
* Maybe: transparent blocks.

## Usage

For the library

```toml
[dependencies]
fastnbt = "0.4.0"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```
