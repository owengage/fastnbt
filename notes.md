# Notes

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

