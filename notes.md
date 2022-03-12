# Notes

# v2

[x] get owned NBT arrays working, also within Value.
[x] get borrowed NBT arrays working for deserialize
[ ] get borrowed NBT arrays working for serialize
[ ] removed deref from nbt arrays
[ ] make sure borrowed/owned interface the same for arrays
[ ] get value working
[ ] Run fuzzer again
[ ] Remove old array deserializing
[ ] Tighten up borrowing bytes/strings.
[ ] anvil: make it work with old chunks again (fixed it to 1.18 only)
 
I don't need to abandon the JavaChunk enum just because I'm abandoning the
deserializer! I can just make the chunk_from_bytes function return the enum
(that doesn't implement deserialize). I could even have JavaChunk::from_bytes.

Abandonded trying to make a enum that can deserialize any chunk over many
versions. It's simply not compatible with the 'mapping into the serde model'
that now occurs for NBT arrays. In particular it seems because I use borrowed
str/bytes for them. Before when it was a sequence the deserializer for the serde
content thing using in untagged enums could figure stuff out. Now it can't.
Turns out it would never worked for borrowed NBT arrays.

https://github.com/serde-rs/serde/issues/1183
https://github.com/serde-rs/json/issues/497
https://github.com/serde-rs/serde/pull/1354

# serializer

Revealed issues with Value type. Need to find equivalent way to do the strict_x
for a vec of those types. serde_as looks like it might be able to do what we
want, but might require changing how I do the strict things.

Fuzzer also revealing more bugs!

MAybe the value type needs to not be an untagged enum. The serde_json Value can
do this somehow, maybe I should explore how that happens? How json handles maps
might be informative, since it somehow detects if it's an infinite precision
number or a normal JSON object.

```
cargo +nightly fuzz run serialize_value
```

# 1.18 hole problem

If I put calculate heights on, it somewhat fixes the issues, except that water
will be missing from some of the erroring chunks. I'd guess that these are solid
blocks of water that have empty data with a palette of 1 or something?

Calculate heights is also not working properly. Gives the height without water
it seems?

# 1.18 chunk format change details

Both biomes and block data is now stored in a `block_states` structure with a
`data` and `palette` field. `data` is like the old `BlockStates` field, and
`palette` is like the old field of the same name.

The `data` is the same, except the lower limit of 4 bits per block does not
apply for the biomes, often the `data` can be 1 bit per block in the case of
only 2 biomes in the section.

`data` may be missing entirely if the `palette` is only 1 item. This seems to
just imply the entire section is that block/biome.

# 1.18 changes

This update brings in some significant changes to the chunk format, ones
relevant to fastanvil are:

* `Level` is gone, with all it's fields moved up into the main chunk compound.
* Names of fields has moved to `snake_case`.
* `BlockStates` and `Palette` have moved into a dedicated container structure.
* `Biomes` is moved to a similar palette/container structure as `BlockStates`.

Conversion to 1.18 chunks is done on a per-chunk basis. So a region file can
contain mixed chunk versions. This means we need to handle the type of chunk at
the chunk level. We cannot 'flip' our processing based on the region file itself.
This awkwardly means we might need an extra 'if' for every function call into a
chunk, or some `dyn` action to virtually dispatch based on the real chunk
version. Neither solution seems particularly pretty to me.

A new 'section' looks like this:

```
"sections": List(
  [
      Compound(
          {
              "biomes": Compound(
                  {
                      "palette": List(
                          [
                              String(
                                  "minecraft:forest",
                              ),
                          ],
                      ),
                  },
              ),
              "Y": Byte(
                  -4,
              ),
              "block_states": Compound(
                  {
                      "data": LongArray(
                          LongArray {
                              tag: LongArray,
                              data: [],
                          },
                      ),
                      "palette": List(
                          [
                              Compound(
                                  {
                                      "Name": String(
                                          "minecraft:stone",
                                      ),
                                  },
                              ),
                          ],
                      ),
                  },
              ),
          },
      ),
    ),
  ]
)
```

The `biomes` here doesn't contain any data. The palette is only one item
however, so it's clear that the entire section must be the same biome. Here's
another that looks unusual:

```
"biomes": Compound(
    {
        "palette": List(
            [
                String(
                    "minecraft:dark_forest",
                ),
                String(
                    "minecraft:lush_caves",
                ),
            ],
        ),
        "data": LongArray(
            LongArray {
                tag: LongArray,
                data: [
                    4222128945627136,
                ],
            },
        ),
    },
),
```

In binary, that data is `1111000000000000000100000000000000000000000000000000`.
If we assume biomes are still encoded in 4x4x4 cubes, then each 'layer' of a
biome will be 4x4=16. It looks like each bit here might be representing a 4x4x4
cube.

In the blockstates packed data the number of bits per 'block' was a minimum of
four. It looks like this might not be the case for biomes. It's probably worth
checking if it has also changed for the normal blockstates.

Example from a 1.16 world blockstate number is 37336289389873219, in binary this
looks like `10000100101001010010100001001010010100001000100001000011` and is
parsed from least significant bit. The palette with this was length 20, so
requires 5 bits per block. So we break it up:

```
1 00001 00101 00101 00101 00001 00101 00101 00001 00010 00010 00011
```

The presence of a few 00001's helps convince us that this is correct. Let's look
at some blockstate data from a new world.

```
Block state:
palette len 8, so 3 bits would be sufficient, but min 4?
1229782938247303441
0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001

palette 7, 3 bits, but still looks like 4.
2459565876494606882
0010 0010 0010 0010 0010 0010 0010 0010 0010 0010 0010 0010 0010 0010 0010 0010
```

Looks like blockstates are still a minimum of 4 bits per block. Amazing. Maybe
it's special cased for biomes, or maybe it's special cased for a palette size of
2? Hard to tell. Maybe generating a flatworld would help.


```
Block state, super flat world with just a layer of bedrock
Palette len 2
1229782938247303441
0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001 0001
```

So looks like it is indeed special cased purely for biomes. Amazing. What
happens if there are more than 2 biomes in a section? Searching through the data
for a biome palette with more than 2 items I get some examples: 

```
{"palette": 
    List([
        String("minecraft:river"), 
        String("minecraft:lush_caves"), 
        String("minecraft:dark_forest")
    ]), 
    "data": LongArray(LongArray { tag: LongArray, data: [
        549755813952, 
        549755814016] })
}
```

Given that we need to encode 64 biomes (4x4x4 cubes), and there's only 2 64-bit
integers here, it seems that the biomes are being encoded 2 bits per biome.

It really seems that for the biome construct it will go down to a single bit per
block, but on blockstates it's a minimum of 4. Maybe this is a historical thing,
maybe it will change, maybe it's intentional based on performance measurements.

# Custom world heights

Having to use a 'null terminated' chunk to determine world depth.

Need to get rid of MIN_Y and MAX_Y.

Grass seems broken too? Sometimes? This is beecause biomes can now be larger

## Reversible Value type

I'm not sure that making a serializable and deserializable Value type is doable.
The main issue I'm seeing it that the deserializer logically produces serdes
data model. This model can encode a sequence of i32 for example, but there are
two distinct ways of decoding this in NBT. It could be a list that happens to
contain i32s, or it could be a IntArray.

I don't think there is a way to encode this extra information in the model. If
we assumed that every sequence of i32 is an IntArray, then deserializing a
List(i32) and serializing the result would not give us the original NBT.

Could dedicated types help? If I have an IntArray type, can I capture the extra
information? I think the information we need it lost by this point. If we
implement Deserialize we get to create a visitor for serde's data model, which
doesn't have the information we need.

We can make the deserializer weird and fake some structure in serde's model to
capture this extra information. Apparently [TOML does this for datetimes] which
I found through this [advanced serde] issue.

This would probably mean mapping the `Array` types to a serde 'map':

```
Map {
  tag: "IntArray",
  data: Seq<i32>,
}
```

With this mapping, a future serializer then has enough information to convert
this back to the correct NBT data.

This would prevent users from using the obvious type when deserializing though.
A users type would expect to be able to put `Vec<i32>` then the NBT data is a
IntArray.

[TOML does this for datetimes]: https://github.com/alexcrichton/toml-rs/blob/master/src/datetime.rs
[advanced serde]: https://github.com/serde-rs/serde/issues/1041

## Performance 

* Chunks seem to always store every section, but sometimes a section is empty,
  denoting a section of air. We currently materialise a block of air in the
  chunk.block() function when this happens, but we could optimise heightmap
  calculation if we take advantage of this explicitly. Maybe a dedicated 'skip
  air' function on a chunk trait.
* [x] `chunk.block()` is pretty inefficient, it's highly used and clones the block,
  which can be many allocations in the case of a block with properties. Maybe
  some interface that lets us pass a closure, that is then called with a refence
  to the existing block?
* Try not unpacking heights/biomes etc, especially for 1.16 onwards as the
  process of getting a value is reasonably straight forward.
* [x] A custom `sections` type that takes care of the arrangement automatically,
  rather than maintaining a seperate sec_map variable. It's super error prone
  atm.
* [x] I changed chunks to be implemented through dynamic dispatch. This means that
  the `block` method on the chunk is virtual. This *might* be a lost
  opportunity, it might be much faster if this was non-virtual. Flamegraph shows
  that half the time spent in this method isn't attributed to anything. I
  imagine that time is the virtual call not showing up.

### Ref block

Change `chunk.block()` to return `Option<&Block>` rather than `Option<Block>`,
saves a reasonably complicated copy.

Pre times for cliff-side: 19.3, 11.3, 11.1, 10.8, 10.6, 10.9
After ref block: 16.3, 9.5, 9.5, 9.4, 9.4, 9.3, 9.3

Definite improvement it seems.

## TODOs

- [x] Top-down shading
- [x] General transparency
- [x] Ocean transparency
- [ ] Kelp visible.
- [ ] Coral under the water!
- [ ] Make PackedBits self contained.
- [ ] Handle invalid region files more gracefully. eg empty file.

### Maybe

These are things that would require serious effort to do correctly, or are not
very interesting to me. Or both.

- [ ] Biome blending. Will require every chunk around the current chunk. Which
  in turn requires all the regions around a region.
- [ ] Top-down shading under transparent ceilings. Not even sure how
  feasible/logical this one is.

