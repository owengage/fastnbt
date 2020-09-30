# fastnbt

Documentation: [docs.rs](https://docs.rs/crate/fastnbt)

A fast (or trying to be!) parser for *Minecraft: Java Edition*'s NBT and Anvil formats.

Supports Serde deserialization. Implemented in a way to try and avoid memory allocations. Strings can be deserialized as `&str`, as well as deserialising the block states of chunks to `&[u8]`.

Aim to support only the latest version of Minecraft. Works with 1.16 worlds at the moment. Endevour to support old chunks in 1.16 worlds, but not extracting textures from older versions due to the added complexity it would require.

The `anvil` binary can render your world leveraging all of your CPU. My 1.5 GiB world is fully rendered to a 14000x14000 PNG in about 4.5 seconds. What about yours?

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

![alt rendered map](demo.png)

# Goals

### Full palette

I currently can extract textures for 657 out of 764 blockstates. I need to implement things like stairs, logs, and rails, cactus etc.

### Advanced state

If I render wheat, for example, I just render all wheat at a particular growth stage. I could extract more information from the chunks and render more exact state.

### Other

* Modify palette colour based on height?
* Change to visitor-based parser to avoid allocation of Array tags when not needed.
* Maybe: some sort of interactive map. WASM?
* Maybe: transparent blocks.

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

## Other notes

### Deserialisation

To make life easier it's going to make sense to [implement a serde Deserializer for NBT](https://serde.rs/impl-deserializer.html). It looks like NBT is a self describing format by serde's definitions, and that the macro `forward_to_deserialize_any!` is going to be very relevant to me.

Look for the `de::Deserializer` impl in the link, specifically the `deserialize_any` function.