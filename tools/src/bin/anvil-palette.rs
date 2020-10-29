use fastanvil::{
    tex::{Blockstate, Model, Render, Renderer, Texture},
    Rgba,
};
use flate2::write::GzEncoder;
use std::error::Error;
use std::path::Path;
use std::{collections::HashMap, fmt::Display};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct ErrorMessage(&'static str);
impl std::error::Error for ErrorMessage {}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn avg_colour(rgba_data: &[u8]) -> Result<Rgba> {
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

    Ok([
        (avg[0] / count as f64).sqrt() as u8,
        (avg[1] / count as f64).sqrt() as u8,
        (avg[2] / count as f64).sqrt() as u8,
        255,
    ])
}

fn load_texture(path: &Path) -> Result<Texture> {
    let img = image::open(path)?;
    let img = img.to_rgba();

    if img.dimensions() == (16, 16) {
        Ok(img.into_raw())
    } else {
        Err(Box::new(ErrorMessage("texture was not 16 by 16")))
    }
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

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let root = Path::new(&args[0]);
    let assets = root.to_owned().join("assets").join("minecraft");

    let textures = load_textures(&assets.join("textures").join("block"))?;
    let blockstates = load_blockstates(&assets.join("blockstates"))?;
    let models = load_models(&assets.join("models").join("block"))?;

    let mut renderer = Renderer::new(blockstates.clone(), models.clone(), textures);
    let mut failed = 0;
    let mut success = 0;

    let mut palette = HashMap::new();

    for name in blockstates
        .keys()
        .filter(|name| args.get(1).map(|s| s == *name).unwrap_or(true))
    {
        let bs = &blockstates[name];

        match bs {
            Blockstate::Variants(vars) => {
                for (props, _) in vars {
                    let res = renderer.get_top(name, props);
                    match res {
                        Ok(texture) => {
                            let col = avg_colour(texture.as_slice())?;
                            palette.insert((*name).clone() + "|" + props, col);

                            success += 1;
                        }
                        Err(e) => {
                            println!("did not understand: {:?}", e);
                            failed += 1;
                        }
                    };
                }
            }
            _ => continue,
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
        "succeeded in understanding {} of {} possible blocks (failed on {})",
        success,
        success + failed,
        failed,
    );
    Ok(())
}
