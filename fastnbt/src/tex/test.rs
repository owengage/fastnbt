use super::*;
use serde_json;

fn cube_model() -> Model {
    serde_json::from_str(
        r##"
        {
            "parent": "block/block",
            "elements": [
                {   "from": [ 0, 0, 0 ],
                    "to": [ 16, 16, 16 ],
                    "faces": {
                        "down":  { "texture": "#down", "cullface": "down" },
                        "up":    { "texture": "#up", "cullface": "up" },
                        "north": { "texture": "#north", "cullface": "north" },
                        "south": { "texture": "#south", "cullface": "south" },
                        "west":  { "texture": "#west", "cullface": "west" },
                        "east":  { "texture": "#east", "cullface": "east" }
                    }
                }
            ]
        }
        "##,
    )
    .unwrap()
}

fn cube_all_model() -> Model {
    serde_json::from_str(
        r##"
        {
            "parent": "block/cube",
            "textures": {
                "particle": "#all",
                "down": "#all",
                "up": "#all",
                "north": "#all",
                "east": "#all",
                "south": "#all",
                "west": "#all"
            }
        }   
        "##,
    )
    .unwrap()
}

fn cobblestone_model() -> Model {
    serde_json::from_str(
        r##"
        {
            "parent": "minecraft:block/cube_all",
            "textures": {
                "all": "minecraft:block/cobblestone"
            }
        }
        "##,
    )
    .unwrap()
}

fn cobblestone_blockstate() -> Blockstate {
    serde_json::from_str(
        r##"
        {
            "variants": {
            "": {
                "model": "minecraft:block/cobblestone"
            }
            }
        }
        "##,
    )
    .unwrap()
}

fn cobblestone_texture() -> Texture {
    [1; 1024]
}
#[test]
fn cobblestone() {
    let blockstates = vec![("minecraft:cobblestone".to_owned(), cobblestone_blockstate())]
        .into_iter()
        .collect();

    let models = vec![
        (
            "minecraft:block/cobblestone".to_owned(),
            cobblestone_model(),
        ),
        ("block/cube".to_owned(), cube_model()),
        ("minecraft:block/cube_all".to_owned(), cube_all_model()),
    ]
    .into_iter()
    .collect();

    let textures = vec![(
        "minecraft:block/cobblestone".to_owned(),
        cobblestone_texture(),
    )]
    .into_iter()
    .collect();

    let mut renderer = Renderer::new(blockstates, models, textures);
    let tex = renderer.get_top("minecraft:cobblestone", "").unwrap();

    assert_eq!(tex, cobblestone_texture());
}
