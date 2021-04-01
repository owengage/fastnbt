# Notes

## Performance 

* Chunks seem to always store every section, but sometimes a section is empty,
  denoting a section of air. We currently materialise a block of air in the
  chunk.block() function when this happens, but we could optimise heightmap
  calculation if we take advantage of this explicitly. Maybe a dedicated 'skip
  air' function on a chunk trait.
* `chunk.block()` is pretty inefficient, it's highly used and clones the block,
  which can be many allocations in the case of a block with properties. Maybe
  some interface that lets us pass a closure, that is then called with a refence
  to the existing block?

## TODOs

- [x] Top-down shading
- [x] General transparency
- [x] Ocean transparency
- [ ] Kelp visible.
- [ ] Coral under the water!

### Maybe

These are things that would require serious effort to do correctly, or are not
very interesting to me. Or both.

- [ ] Biome blending. Will require every chunk around the current chunk. Which
  in turn requires all the regions around a region.
- [ ] Top-down shading under transparent ceilings. Not even sure how
  feasible/logical this one is.

