# fastnbt project

Crate documentation:
* [docs.rs/crate/fastnbt](https://docs.rs/crate/fastnbt)
* [docs.rs/crate/fastanvil](https://docs.rs/crate/fastanvil)

This repository contains multiple related projects.

* [fastnbt](fastnbt/README.md): Fast (or trying to be!) deserializer and parser for *Minecraft: Java Edition*'s NBT data format.
* fastanvil: For rendering Minecraft worlds to maps.
* fastnbt-tools: Various tools for NBT/Anvil, notably a map renderer.
* anvil-wasm: An entirely in-the-browser map renderer. Demo at [owengage.com/anvil](https://owengage.com/anvil).

Aim to support only the latest version of Minecraft. Works with 1.16 worlds at the moment. Endevour to support old chunks in 1.16 worlds, but not extracting textures from older versions due to the added complexity it would require.

The `anvil` binary can render your world leveraging all of your CPU. My 3.2 GiB world with 271k chunks is fully rendered to a 14000x17000 PNG in about 7 seconds. What about yours?

![alt rendered map](demo.png)

# Render your map

```bash
cargo install fastnbt-tools

# Extract a minecraft version for getting the palette out.
# This will be simpler in future.
# on macOS: `pushd ~/Library/Application\ Support/minecraft/versions/1.16.1/ && mkdir unpacked && cd unpacked`
pushd ~/.minecraft/versions/1.16.1/ && mkdir unpacked && cd unpacked
unzip ../1.16.1.jar
popd

# Create a palette to render with
anvil-palette ~/.minecraft/versions/1.16.1/unpacked 

 # render entire overworld
anvil render ~/path/to/world-dir --palette=palette.tar

# render entire end
anvil render ~/path/to/world-dir --dimension=end --palette=palette.tar 

# render 6 by 6 regions around 0,0.
anvil render ~/path/to/world-dir --size=6,6  --palette=palette.tar 

# render 10 by 10 offset by x: -4, z: 10.
anvil render ~/path/to/world-dir --size=10,10 --offset=-4,10  --palette=palette.tar 
```

# Development priorities

These are the proirities for the project when it comes to development. Ideally we never sacrifice an earlier priority for the sake of a later one.

1. Correctness. Worlds are rendered as accurately as possible.
2. Speed. Worlds are rendered as fast as possible.
3. Memory. Worlds are rendered without sucking up RAM.

## Usage

For the libraries

```toml
[dependencies]
fastnbt = "0.12"
fastanvil = "0.12"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```