# fastnbt

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

A fast (or trying to be!) parser for *Minecraft: Java Edition*'s NBT and Anvil formats.

Uses Rayon to utilise all cores of the machine. On a Ryzen 3600X 6-core, with a reasonably complex world, it can render a map of  256 *regions* in 9 seconds. That's 262k chunks, about 30k chunks/s.

```bash
anvil render ~/path/to/world-dir --min-x=-1 --min-z=-1 --max-x=1 --max-z=1
```

![alt rendered map](map.png)

## TODO

* Full palette for block-based world rendering.
* Modify palette colour based on height.
* Change to visitor-based parser to avoid allocation of Array tags when not needed.
* Test on Windows.
* Use newtypes idiom for the various co-ordinate types for safety.

## Usage

For the library

```toml
[dependencies]
fastnbt = "0.2.0"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```
