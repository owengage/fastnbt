use fastnbt::tex::{Blockstate, Model, Renderer, Variant, Variants};
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

fn load_textures(path: &Path) -> Result<HashMap<String, [u8; 3]>> {
    let mut tex = HashMap::new();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().ok_or("invalid ext")?.to_string_lossy() == "png" {
            let colour = avg_colour(&path)?;

            tex.insert(
                "minecraft:block/".to_owned()
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

    Err("did not understand model")?
}

fn find_texture(state: &Blockstate, models: &HashMap<String, Model>) -> Result<String> {
    if let Blockstate::Variants(ref v) = state {
        for (_vname, variant) in v {
            return match variant {
                Variants::Single(ref var) => find_model_in_variant(var, models),
                Variants::Many(vars) => find_model_in_variant(&vars[0], models),
            };
        }
    };

    Err("did not understand blockstate")?
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let root = Path::new(&args[0]);
    let assets = root.to_owned().join("assets").join("minecraft");

    let blockstates = load_blockstates(&assets.join("blockstates"))?;
    let models = load_models(&assets.join("models").join("block"))?;
    //let textures = load_textures(&assets.join("textures").join("block"))?;

    let renderer = Renderer::new(blockstates, models.clone(), HashMap::new());
    let mut failed = 0;
    let mut success = 0;

    for name in models.keys() {
        let res = renderer.flatten_model(name);
        match res {
            Ok(model) => {
                eprintln!("{}: {:#?}", name, model);
                success += 1
            }
            Err(_) => failed += 1,
        }
    }

    eprintln!("success {:?}, failed {:?}", success, failed);
    Ok(())
}
