use fastanvil::{
    tex::{Blockstate, Model, Render, Renderer, Texture},
    Rgba,
};
use flate2::write::GzEncoder;
use std::error::Error;
use std::path::Path;
use std::{collections::HashMap, fmt::Display};

use regex::Regex;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct ErrorMessage(&'static str);
impl std::error::Error for ErrorMessage {}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn avg_colour(rgba_data: &[u8]) -> Rgba {
    let mut avg = [0f64; 3];
    let mut count = 0;

    for p in rgba_data.chunks(4) {
        // alpha is reasonable.
        if p[3] > 128 {
            avg[0] = avg[0] + ((p[0] as u64) * (p[0] as u64)) as f64;
            avg[1] = avg[1] + ((p[1] as u64) * (p[1] as u64)) as f64;
            avg[2] = avg[2] + ((p[2] as u64) * (p[2] as u64)) as f64;
            count = count + 1;
        }
    }

    [
        (avg[0] / count as f64).sqrt() as u8,
        (avg[1] / count as f64).sqrt() as u8,
        (avg[2] / count as f64).sqrt() as u8,
        255,
    ]
}

fn load_texture(path: &Path) -> Result<Texture> {
    let img = image::open(path)?;
    let img = img.to_rgba8();

    //if img.dimensions() == (16, 16) {
    Ok(img.into_raw())
    // } else {
    //     Err(Box::new(ErrorMessage("texture was not 16 by 16")))
    // }
}

fn load_blockstates(blockstates_path: &Path) -> Result<HashMap<String, Blockstate>> {
    let mut blockstates = HashMap::<String, Blockstate>::new();

    for entry in std::fs::read_dir(blockstates_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let json = std::fs::read_to_string(&path)?;

            let json: Blockstate = serde_json::from_str(&json)?;
            blockstates.insert(
                "minecraft:".to_owned()
                    + path
                        .file_stem()
                        .ok_or(format!("invalid file name: {}", path.display()))?
                        .to_str()
                        .ok_or(format!("nonunicode file name: {}", path.display()))?,
                json,
            );
        }
    }

    Ok(blockstates)
}

fn load_models(path: &Path) -> Result<HashMap<String, Model>> {
    let mut models = HashMap::<String, Model>::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let json = std::fs::read_to_string(&path)?;

            let json: Model = serde_json::from_str(&json)?;
            models.insert(
                "minecraft:block/".to_owned()
                    + path
                        .file_stem()
                        .ok_or(format!("invalid file name: {}", path.display()))?
                        .to_str()
                        .ok_or(format!("nonunicode file name: {}", path.display()))?,
                json,
            );
        }
    }

    Ok(models)
}

fn load_textures(path: &Path) -> Result<HashMap<String, Texture>> {
    let mut tex = HashMap::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().ok_or("invalid ext")?.to_string_lossy() == "png" {
            let texture = load_texture(&path);

            match texture {
                Err(_) => continue,
                Ok(texture) => tex.insert(
                    "minecraft:block/".to_owned()
                        + path
                            .file_stem()
                            .ok_or(format!("invalid file name: {}", path.display()))?
                            .to_str()
                            .ok_or(format!("nonunicode file name: {}", path.display()))?,
                    texture,
                ),
            };
        }
    }

    Ok(tex)
}

#[derive(Debug)]
struct RegexMapping {
    blockstate: Regex,
    texture_template: &'static str,
}

impl RegexMapping {
    fn apply(&self, blockstate: &str) -> Option<String> {
        let caps = self.blockstate.captures(blockstate)?;

        let mut i = 1;
        let mut tex = self.texture_template.to_string();

        for cap in caps.iter().skip(1) {
            let cap = match cap {
                Some(cap) => cap,
                None => continue,
            };

            tex = tex.replace(&format!("${}", i), cap.into());
            i += 1;
        }

        Some(tex)
    }
}

pub fn make_palette(mc_jar_path: &Path) -> Result<()> {
    let assets = mc_jar_path.to_owned().join("assets").join("minecraft");

    let textures = load_textures(&assets.join("textures").join("block"))?;
    let blockstates = load_blockstates(&assets.join("blockstates"))?;
    let models = load_models(&assets.join("models").join("block"))?;

    let mut renderer = Renderer::new(blockstates.clone(), models.clone(), textures.clone());
    let mut failed = 0;
    let mut mapped = 0;
    let mut success = 0;

    let mappings = vec![
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_fence").unwrap(),
            texture_template: "minecraft:block/$1_planks",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_wall(_sign)?").unwrap(),
            texture_template: "minecraft:block/$1_planks",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_wall(_sign)?").unwrap(),
            texture_template: "minecraft:block/$1",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_glazed_terracotta").unwrap(),
            texture_template: "minecraft:block/$1_glazed_terracotta",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_mushroom_block").unwrap(),
            texture_template: "minecraft:block/$1_mushroom_block",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:wheat").unwrap(),
            texture_template: "minecraft:block/wheat_stage7",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:carrots").unwrap(),
            texture_template: "minecraft:block/carrots_stage3",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:poppy").unwrap(),
            texture_template: "minecraft:block/poppy",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:daisy").unwrap(),
            texture_template: "minecraft:block/daisy",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:dandelion").unwrap(),
            texture_template: "minecraft:block/dandelion",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:oxeye_daisy").unwrap(),
            texture_template: "minecraft:block/oxeye_daisy",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:azure_bluet").unwrap(),
            texture_template: "minecraft:block/azure_bluet",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:lava").unwrap(),
            texture_template: "minecraft:block/lava_still",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:dead_bush").unwrap(),
            texture_template: "minecraft:block/dead_bush",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_tulip").unwrap(),
            texture_template: "minecraft:block/$1_tulip",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:allium").unwrap(),
            texture_template: "minecraft:block/allium",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:cornflower").unwrap(),
            texture_template: "minecraft:block/cornflower",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:lily_of_the_valley").unwrap(),
            texture_template: "minecraft:block/lily_of_the_valley",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:sugar_cane").unwrap(),
            texture_template: "minecraft:block/sugar_cane",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:sunflower").unwrap(),
            texture_template: "minecraft:block/sunflower_front",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:peony").unwrap(),
            texture_template: "minecraft:block/peony_top",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:rose_bush").unwrap(),
            texture_template: "minecraft:block/rose_bush_top",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:lilac").unwrap(),
            texture_template: "minecraft:block/lilac_top",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_orchid").unwrap(),
            texture_template: "minecraft:block/$1_orchid",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:sweet_berry_bush").unwrap(),
            texture_template: "minecraft:block/sweet_berry_bush_stage3",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(.+)_mushroom").unwrap(),
            texture_template: "minecraft:block/$1_mushroom",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:potatoes").unwrap(),
            texture_template: "minecraft:block/potatoes_stage3",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:(\w+)_sapling").unwrap(),
            texture_template: "minecraft:block/$1_sapling",
        },
        RegexMapping {
            blockstate: Regex::new(r"minecraft:tripwire").unwrap(),
            texture_template: "minecraft:block/tripwire",
        },
    ];

    let mut palette = HashMap::new();

    let mut try_mapping = |mapping: &RegexMapping, blockstate: String| {
        if let Some(tex) = mapping.apply(&blockstate) {
            let texture = textures.get(&tex);
            println!("map: {:?} to {}, {:?}", mapping, blockstate, tex);

            match texture {
                Some(texture) => {
                    println!("mapped {} to {}", blockstate, tex);
                    mapped += 1;
                    let col = avg_colour(texture.as_slice());
                    return Some(col);
                }
                None => {}
            }
        }

        None
    };

    let mut try_mappings = |blockstate: String| {
        let c = mappings
            .iter()
            .map(|mapping| try_mapping(mapping, blockstate.clone()))
            .find_map(|col| col);

        if c.is_none() {
            println!("did not understand: {:?}", blockstate);
            failed += 1;
        }

        c
    };

    for name in blockstates.keys() {
        let bs = &blockstates[name];

        match bs {
            Blockstate::Variants(vars) => {
                for (props, _) in vars {
                    let res = renderer.get_top(name, props);
                    match res {
                        Ok(texture) => {
                            let col = avg_colour(texture.as_slice());

                            // We want to add the pipe if the props are anything
                            // but empty.
                            let description =
                                (*name).clone() + if *props == "" { "" } else { "|" } + props;

                            palette.insert(description, col);
                            success += 1;
                        }
                        Err(e) => {
                            try_mappings((*name).clone()).map(|c| {
                                palette.insert((*name).clone(), c);
                                eprintln!("mapped {}", *name);
                            });
                        }
                    };
                }
            }
            Blockstate::Multipart(_) => {
                try_mappings((*name).clone()).map(|c| {
                    palette.insert((*name).clone(), c);
                });
            }
        }
    }

    let f = std::fs::File::create("palette.tar.gz")?;
    let f = GzEncoder::new(f, Default::default());

    let mut ar = tar::Builder::new(f);

    let grass_colourmap = &assets.join("textures").join("colormap").join("grass.png");
    ar.append_file(
        "grass-colourmap.png",
        &mut std::fs::File::open(grass_colourmap)?,
    )?;

    let foliage_colourmap = &assets.join("textures").join("colormap").join("foliage.png");
    ar.append_file(
        "foliage-colourmap.png",
        &mut std::fs::File::open(foliage_colourmap)?,
    )?;

    let palette_data = serde_json::to_vec(&palette)?;
    let mut header = tar::Header::new_gnu();
    header.set_size(palette_data.len() as u64);
    header.set_cksum();
    header.set_mode(0o666);
    ar.append_data(&mut header, "blockstates.json", palette_data.as_slice())?;

    // finishes the archive.
    let f = ar.into_inner()?;
    f.finish()?;

    println!(
        "succeeded in understanding {} of {} possible blocks (mapped {}, failed on {})",
        success,
        success + failed,
        mapped,
        failed,
    );

    Ok(())
}
