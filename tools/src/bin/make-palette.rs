use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn avg_colour(path: &Path) -> Result<[u8; 3]> {
    let img = image::open(path)?;
    let img = img.to_rgba();
    //.ok_or(format!("image was not RGBA: {}", path.display()))?;

    let mut avg = [0f64; 3];
    let mut count = 0;

    for p in img.pixels() {
        // alpha is reasonable.
        if p.0[3] > 128 {
            avg[0] = avg[0] + ((p.0[0] as u64) * (p.0[0] as u64)) as f64;
            avg[1] = avg[1] + ((p.0[1] as u64) * (p.0[1] as u64)) as f64;
            avg[2] = avg[2] + ((p.0[2] as u64) * (p.0[2] as u64)) as f64;
            count = count + 1;
        }
    }

    Ok([
        (avg[0] / count as f64).sqrt() as u8,
        (avg[1] / count as f64).sqrt() as u8,
        (avg[2] / count as f64).sqrt() as u8,
    ])
}

#[derive(Deserialize, Debug)]
struct Variant {
    model: String,
    x: Option<usize>,
    y: Option<usize>,
    uvlock: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum Variants {
    Single(Variant),
    Many(Vec<Variant>),
}

#[derive(Deserialize, Debug)]
struct Blockstate {
    variants: Option<HashMap<String, Variants>>,
    multipart: Option<Vec<Part>>,
}

#[derive(Deserialize, Debug)]
struct Part {
    when: Option<serde_json::Value>,
    apply: Variants,
}

#[derive(Deserialize, Debug)]
struct Model {
    parent: Option<String>,
    textures: Option<HashMap<String, String>>,
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
                "block/".to_owned()
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

fn load_textures(path: &Path) -> Result<HashMap<String, [u8; 3]>> {
    let mut tex = HashMap::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().ok_or("invalid ext")?.to_string_lossy() == "png" {
            let colour = avg_colour(&path)?;

            tex.insert(
                "block/".to_owned()
                    + path
                        .file_stem()
                        .ok_or(format!("invalid file name: {}", path.display()))?
                        .to_str()
                        .ok_or(format!("nonunicode file name: {}", path.display()))?,
                colour,
            );
        }
    }

    Ok(tex)
}

fn find_model_in_variant(var: &Variant, models: &HashMap<String, Model>) -> Result<String> {
    let model = models
        .get(&var.model)
        .ok_or(format!("did not find {}", &var.model))?;

    if let Some(ref textures) = model.textures {
        if let Some(t) = textures.get("all") {
            return Ok(t.clone());
        }

        if let Some(t) = textures.get("top") {
            return Ok(t.clone());
        }

        if let Some(t) = textures.get("plant") {
            return Ok(t.clone());
        }

        if let Some(t) = textures.get("texture") {
            return Ok(t.clone());
        }

        if let Some(t) = textures.get("particle") {
            return Ok(t.clone());
        }

        if let Some(t) = textures.get("fan") {
            return Ok(t.clone());
        }
        if let Some(t) = textures.get("cross") {
            return Ok(t.clone());
        }
        if let Some(t) = textures.get("side") {
            // eg logs, has side and end.
            return Ok(t.clone());
        }
        if let Some(t) = textures.get("crop") {
            // eg wheat
            return Ok(t.clone());
        }
    }

    Err("no texture")?
}

fn find_texture(state: &Blockstate, models: &HashMap<String, Model>) -> Result<String> {
    if let Some(ref v) = state.variants {
        for (_vname, variant) in v {
            return match variant {
                Variants::Single(ref var) => find_model_in_variant(var, models),
                Variants::Many(vars) => find_model_in_variant(&vars[0], models),
            };
        }
    };

    Err("no texture found")?
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let root = Path::new(&args[0]);
    let assets = root.to_owned().join("assets").join("minecraft");

    let blockstates = load_blockstates(&assets.join("blockstates"))?;
    let models = load_models(&assets.join("models").join("block"))?;

    let mut textured_blocks = HashMap::<String, String>::new();

    for (name, state) in &blockstates {
        let texture = find_texture(state, &models);

        if let Ok(texture) = texture {
            textured_blocks.insert(name.clone(), texture.clone());
        }
    }

    eprintln!("total blockstates: {}", blockstates.len());
    eprintln!("textured blockstates: {}", textured_blocks.len());

    let textures = load_textures(&assets.join("textures").join("block"))?;

    let mut palette = HashMap::new();

    for (id, tex) in textured_blocks {
        palette.insert(id, textures.get(&tex).unwrap_or(&[0u8, 255, 255]));
    }

    let f = std::fs::File::create("palette.tar")?;
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
    ar.into_inner()?;
    Ok(())
}
