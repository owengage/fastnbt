use anyhow::Context;
use fastanvil::{RenderedPalette, Rgba};
use flate2::read::GzDecoder;

pub fn get_palette() -> anyhow::Result<RenderedPalette> {
    let data = include_bytes!("../../../palette.tar.gz");
    let f = GzDecoder::new(data.as_slice());
    let mut ar = tar::Archive::new(f);
    let mut grass = Err(anyhow::format_err!("no grass colour map"));
    let mut foliage = Err(anyhow::format_err!("no foliage colour map"));
    let mut blockstates = Err(anyhow::format_err!("no blockstate palette"));

    for file in ar.entries()? {
        let mut file = file?;
        match file.path()?.to_str().context("invalid path in TAR")? {
            "grass-colourmap.png" => {
                use std::io::Read;
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;

                grass = Ok(
                    image::load(std::io::Cursor::new(buf), image::ImageFormat::Png)?.into_rgba8(),
                );
            }
            "foliage-colourmap.png" => {
                use std::io::Read;
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;

                foliage = Ok(
                    image::load(std::io::Cursor::new(buf), image::ImageFormat::Png)?.into_rgba8(),
                );
            }
            "blockstates.json" => {
                let json: std::collections::HashMap<String, Rgba> = serde_json::from_reader(file)?;
                blockstates = Ok(json);
            }
            _ => {}
        }
    }

    let p = RenderedPalette {
        blockstates: blockstates?,
        grass: grass?,
        foliage: foliage?,
    };

    Ok(p)
}
