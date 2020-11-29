use std::{error::Error, path::Path};

use fastnbt_tools::make_palette;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    make_palette(Path::new(&args[0]))?;

    Ok(())
}
