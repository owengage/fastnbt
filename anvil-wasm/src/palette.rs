use fastanvil::{RenderedPalette, Rgba};
use flate2::read::GzDecoder;
use std::{error::Error, io::Cursor};

const PALETTE: &[u8] = include_bytes!("../palette.tar.gz");

pub fn get_palette() -> Result<RenderedPalette, Box<dyn Error>> {
    let f = Cursor::new(PALETTE);
    let f = GzDecoder::new(f);
    let mut ar = tar::Archive::new(f);
    let mut grass = Err("no grass colour map");
    let mut foliage = Err("no foliage colour map");
    let mut blockstates = Err("no blockstate palette");

    for file in ar.entries()? {
        let mut file = file?;
        match file.path()?.to_str().ok_or("invalid path in TAR")? {
            "grass-colourmap.png" => {
                use std::io::Read;
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;

                grass = Ok(
                    image::load(std::io::Cursor::new(buf), image::ImageFormat::Png)?.into_rgba(),
                );
            }
            "foliage-colourmap.png" => {
                use std::io::Read;
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;

                foliage = Ok(
                    image::load(std::io::Cursor::new(buf), image::ImageFormat::Png)?.into_rgba(),
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
