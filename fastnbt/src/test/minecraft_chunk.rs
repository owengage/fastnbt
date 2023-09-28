use super::resources::{CHUNK_RAW, CHUNK_RAW_WITH_ENTITIES};
use crate::{from_bytes, Value};
use serde::Deserialize;

#[test]
fn unit_variant_enum_for_chunk_status() {
    // From https://minecraft.wiki/w/Chunk_format
    //
    //  Status: Defines the world generation status of this chunk. It is always
    //  one of the following: empty, structure_starts, structure_references,
    //  biomes, noise, surface, carvers, liquid_carvers, features, light, spawn,
    //  or heightmaps.[1]
    #[derive(Deserialize)]
    struct Chunk {
        #[serde(rename = "Level")]
        level: Level,
    }

    #[derive(Deserialize)]
    struct Level {
        #[serde(rename = "Status")]
        status: Status,
    }

    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(rename_all = "snake_case")]
    enum Status {
        Empty,
        StructureStarts,
        StructureReferences,
        Biomes,
        Noise,
        Surface,
        Carvers,
        LiquidCarvers,
        Features,
        Light,
        Spawn,
        Heightmaps,
        Full,
    }

    let chunk: Chunk = from_bytes(CHUNK_RAW).unwrap();
    assert_eq!(Status::Full, chunk.level.status)
}
#[test]
fn chunk_to_value() {
    from_bytes::<Value>(CHUNK_RAW).unwrap();
}

#[test]
fn tile_entities() {
    #[derive(Deserialize)]
    struct Chunk {
        #[serde(rename = "Level")]
        level: Level,
    }

    #[derive(Deserialize)]
    struct Level {
        #[serde(rename = "Entities")]
        entities: Vec<Entity>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(untagged)]
    enum Entity {
        Known(KnownEntity),
        Unknown(Value),
    }

    #[derive(Deserialize, Debug)]
    #[serde(tag = "id")]
    enum KnownEntity {
        #[serde(rename = "minecraft:bat")]
        Bat {
            #[allow(dead_code)]
            #[serde(rename = "BatFlags")]
            bat_flags: i8,
        },

        #[serde(rename = "minecraft:creeper")]
        Creeper {
            #[allow(dead_code)]
            ignited: i8,
        },
    }

    let chunk: Chunk = from_bytes(CHUNK_RAW_WITH_ENTITIES).unwrap();
    let entities = chunk.level.entities;

    println!("{:#?}", entities);
}

#[test]
fn avoiding_alloc_with_chunk() {
    #[derive(Deserialize)]
    struct Chunk<'a> {
        #[serde(rename = "Level")]
        #[serde(borrow)]
        _level: Level<'a>,
    }

    #[derive(Deserialize)]
    struct Level<'a> {
        #[serde(rename = "Sections")]
        #[serde(borrow)]
        pub _sections: Option<Vec<Section<'a>>>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    pub struct Section<'a> {
        #[serde(borrow)]
        pub block_states: Option<&'a [u8]>,
    }

    let _chunk: Chunk = from_bytes(CHUNK_RAW).unwrap();
}
