# Advanced state

We want to accurately paint a pixel on the map. To do this we need to know more
than just what block is placed in the given location. We need to also know what
state that block is in. For example, a redstone torch would look a different
colour depending on whether the torch is lit or not.

We need to pull together information from many places to figure out exactly what
colour a pixel should be. First we need to find the unpack the blockstates in a
section of a chunk. These block states are conceptually just an array of numbers
with some fancy bit-packing. The numbers in this array are actually indicies
into the palette that can be found alongside these block states.

Using the `region-dump` utility, we can see that the block state data might look
something like this:

```
LongArray(Some("BlockStates"), [
    1229782938247303441, 
    1229782938247303441,
    1229782938247303441, ...])
```

and the palette might look something like this:

```
List(Some("Palette"), Compound, 16)
    Compound(None)                        
        String(Some("Name"), "minecraft:air")                    
    CompoundEnd                    
    Compound(None)                        
        String(Some("Name"), "minecraft:bedrock")     
    CompoundEnd                
    Compound(None)      
        String(Some("Name"), "minecraft:stone")       
    CompoundEnd            
    Compound(None)                 
        Compound(Some("Properties"))                     
            String(Some("lit"), "false")     
        CompoundEnd         
        String(Some("Name"), "minecraft:redstone_ore")
    CompoundEnd
    Compound(None)
        String(Some("Name"), "minecraft:gravel")
    CompoundEnd
```

We can see in the palette above that there is a redstone ore. This palette item
has some extra properties to inform us of the state of the block. Here we can
see that the ore is not lit.

This does not tell us how to actually colour a pixel on the map though. Surely
we need the textures. Where are those? This stuff lives in the Minecraft JAR
file itself.

The Minecraft JAR file can be unpacked like a zip file. Within it we find an
assets directory. Searching around we can find a very promising `minecraft/textures/block`
directory.

In this directory we can find two different PNG files, `redstone_torch.png` and
`redstone_torch_off.png`. These show a redstone torch from the side in the on
and off states. How do we connect this to a palette item?

In the `minecraft/blockstates` directory we can find the following for a
redstone torch:

```json
{
  "variants": {
    "lit=false": {
      "model": "minecraft:block/redstone_torch_off"
    },
    "lit=true": {
      "model": "minecraft:block/redstone_torch"
    }
  }
}
```

This seems to show us the various variants that a redstone torch can be in. We
can see that the name of each variant seems to be an encoded version of the
property list. This seems like an interesting way of doing it. What does it look
like for a more complex block, say stairs?

Red netherbrick stairs seems to indeed result in a massive blockstate file,
which I won't repeat here since it's very long. But one of the variants looks
like

```json
{
  "variants": {
    "facing=east,half=bottom,shape=inner_left": {
      "model": "minecraft:block/red_nether_brick_stairs_inner",
      "y": 270,
      "uvlock": true
    },
    ...
  }
}
```

It looks like the properties are listed (in alphabetical order?) as the name of
the variant. It then contains some information. The model looks interesting, and
there happens to be a `models/block` directory too. What's in there?

We can find the mentioned model for redstone torch above:

```json
{
  "parent": "minecraft:block/template_torch",
  "textures": {
    "torch": "minecraft:block/redstone_torch"
  }
}
```

This points us at the texture, meaning we've managed to connect all the way from
the blockstate index to the texture. But what's this `template_torch`?

```json
{
    "ambientocclusion": false,
    "textures": {
        "particle": "#torch"
    },
    "elements": [
        {   "from": [ 7, 0, 7 ],
            "to": [ 9, 10, 9 ],
            "shade": false,
            "faces": {
                "down": { "uv": [ 7, 13, 9, 15 ], "texture": "#torch" },
                "up":   { "uv": [ 7,  6, 9,  8 ], "texture": "#torch" }
            }
        },
        {   "from": [ 7, 0, 0 ],
            "to": [ 9, 16, 16 ],
            "shade": false,
            "faces": {
                "west": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" },
                "east": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" }
            }
        },
        {   "from": [ 0, 0, 7 ],
            "to": [ 16, 16, 9 ],
            "shade": false,
            "faces": {
                "north": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" },
                "south": { "uv": [ 0, 0, 16, 16 ], "texture": "#torch" }
            }
        }
    ]
}
```

Some detailed information about this JSON can be found
[here](https://minecraft.gamepedia.com/Model). The important details for us are:

* `#torch` refers the value of `texture.torch` in the torch model.
* `uv` refers to the coordinates of the texture to use in the format 
  `[x1, y1,  x2, y2]`. If it's absent then these coordinates are worked out from the `from`
  and `to` 3D coordinates instead.
* The `faces` object shows what faces are actually rendered. If the face is not
  mentioned, it is not rendered.

So the process looks like this for a redstone torch:

1. Unpack the block states from the chunk. Get the index for wherever the
   redstone torch is from this.
2. Find the palette item, it will be a redstone torch and have a `lit` property
   set to `true`.
3. Encode the properties and values to produce `lit=true`.
4. Look up this in the variants of the blockstate JSON file. This gives us a
   model of `minecraft:block/redstone_torch`.
5. Look up this model, note that the `torch` texture variable is 
   `minecraft:block/redstone_torch`.
6. Note that the model has a parent. Look up that parent and see that there is
   an elements array.
7. In these elements, find faces that are 'up' since that's what we're drawing.
8. For each of these faces, pull parts of the texture using the `uv` data and
   the mentioned texture variable `#torch`, which we noted was
   `minecraft:block/redstone_torch`.
9. At this point we might want to draw the block below the torch to fill in the
   transparent parts. Or we could stop and average the pixels to get a final
   colour for the pixel.

From this, it seems sensible that the way to draw a map is in two parts. We can
process every single variant of every blockstate, follow this process and
calculate a texture. This will be a *lot* of 16 by 16 textures, each taking at
least 1 kiB to store (we also need to store the encoded properties and name).
This would only be if we want to support transparency, which is a future goal.

How many variants are there?

```bash
fd . ~/.minecraft/versions/1.16.1/unpacked/assets/minecraft/blockstates \
    | xargs jq '.variants | length' \
    | awk '{ sum += $1 } END { print sum; }'
3791
```

So we're probably talking about a collection of textures totalling over 4 MiB.
There are also block states which use a `multipart` way of doing things rather
than variants. There are around 50 block state files for these. So the total
amount of textures produced will be large, but not insurmountable.

# Conclusion

A module that allows you to provide all of this information, and then let you
produce all of these textures would take us closer to being able to render all
blocks accurately.