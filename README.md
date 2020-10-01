# fastnbt

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

A fast (or trying to be!) parser for *Minecraft: Java Edition*'s NBT and Anvil formats.

* Serde based deserialisation into structs from in-memory bytes.
* `Read` based parser for large files while keeping a low memory footprint.
* `fastnbt-tools` contains executables to render your world.

Aim to support only the latest version of Minecraft. Works with 1.16 worlds at the moment. Endevour to support old chunks in 1.16 worlds, but not extracting textures from older versions due to the added complexity it would require.

Serde deserialization is implemented in a way to try and avoid memory allocations. Strings can be deserialized as `&str`, as well as deserialising the block states of chunks to `&[u8]`.

The `anvil` binary can render your world leveraging all of your CPU. My 3.2 GiB world with 271k chunks is fully rendered to a 14000x17000 PNG in about 7 seconds. What about yours?

![alt rendered map](demo.png)

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

1. Correctness. Worlds are rendered as accurately as possible.
2. Speed. Worlds are rendered as fast as possible.
3. Memory. Worlds are rendered without sucking up RAM.

# Goals

## Serde deserialization

This mostly works. I have not yet implemented enums, which will be important for properly parsing the palette to correctly render things like wheat, stairs, etc. See the 'Advanced State' goal.

## WASM based online personal map explorer

It should be possible to compile fastnbt to WASM, and allow players to give access to their region files in the browser with the `File` web API. From there we should be able to render people's worlds (not their seed!) in their browser, with their own computing power.

The current version can render about 30,000/s chunks on my computer using 16 CPU threads, so it would hopefully render fast enough to be usable on peoples computers in the browser, especially if we can somehow leverage multiple cores eg via web workers.

## Full palette

I currently can extract textures for 657 out of 764 blockstates. I need to implement things like stairs, logs, and rails, cactus etc. The horrible megenta colour is a result of failing to extract textures.

## Advanced state

If I render wheat, for example, I just render all wheat at a particular growth stage. I could extract more information from the chunks and render more exact state.

## Other

* Maybe: Modify palette colour based on height?
* Maybe: render blocks below transparent blocks.

## Usage

For the library

```toml
[dependencies]
fastnbt = "0.7.0"
```

For the `anvil` executable

```bash
cargo install fastnbt-tools
```