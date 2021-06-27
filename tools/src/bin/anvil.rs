use clap::{App, Arg, ArgMatches, SubCommand};
use env_logger::Env;
use fastanvil::{render_region, CCoord, HeightMode, RCoord, RegionLoader, Rgba, TopShadeRenderer};
use fastanvil::{Dimension, RenderedPalette};

use fastanvil::RegionFileLoader;
use flate2::read::GzDecoder;
use image;
use log::{error, info};
use rayon::prelude::*;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_coord(coord: &str) -> Option<(isize, isize)> {
    let mut s = coord.split(",");
    let x: isize = s.next()?.parse().ok()?;
    let z: isize = s.next()?.parse().ok()?;
    Some((x, z))
}

fn auto_size(coords: &Vec<(RCoord, RCoord)>) -> Option<Rectangle> {
    if coords.len() == 0 {
        return None;
    }

    let mut bounds = Rectangle {
        xmin: RCoord(isize::MAX),
        zmin: RCoord(isize::MAX),
        xmax: RCoord(isize::MIN),
        zmax: RCoord(isize::MIN),
    };

    for coord in coords {
        bounds.xmin = std::cmp::min(bounds.xmin, coord.0);
        bounds.xmax = std::cmp::max(bounds.xmax, coord.0);
        bounds.zmin = std::cmp::min(bounds.zmin, coord.1);
        bounds.zmax = std::cmp::max(bounds.zmax, coord.1);
    }

    Some(bounds)
}

fn make_bounds(size: (isize, isize), off: (isize, isize)) -> Rectangle {
    Rectangle {
        xmin: RCoord(off.0 - (size.0 + 0) / 2), // size + 1 makes sure that a size of 1,1
        xmax: RCoord(off.0 + (size.0 + 1) / 2), // produces bounds of size 1,1 rather than
        zmin: RCoord(off.1 - (size.1 + 0) / 2), // the 0,0 you would get without it.
        zmax: RCoord(off.1 + (size.1 + 1) / 2),
    }
}

#[derive(Debug)]
struct Rectangle {
    xmin: RCoord,
    xmax: RCoord,
    zmin: RCoord,
    zmax: RCoord,
}

fn get_palette(path: Option<&str>) -> Result<RenderedPalette> {
    let path = match path {
        Some(path) => Path::new(path),
        None => panic!("no palette"),
    };

    let f = std::fs::File::open(path)?;
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

fn render(args: &ArgMatches) -> Result<()> {
    let world: PathBuf = args.value_of("world").unwrap().parse().unwrap();
    let dim: &str = args.value_of("dimension").unwrap();
    let height_mode = match args.is_present("calculate-heights") {
        true => HeightMode::Calculate,
        false => HeightMode::Trust,
    };

    let subpath = match dim {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    let loader = RegionFileLoader::new(world.join(subpath));

    let coords = loader.list()?;

    let bounds = match (args.value_of("size"), args.value_of("offset")) {
        (Some(size), Some(offset)) => {
            make_bounds(parse_coord(size).unwrap(), parse_coord(offset).unwrap())
        }
        (None, _) => auto_size(&coords).unwrap(),
        _ => panic!(),
    };

    info!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;

    let pal = get_palette(args.value_of("palette"))?;

    let region_maps: Vec<_> = coords
        .into_par_iter()
        .filter_map(|coord| {
            let loader = RegionFileLoader::new(world.join(subpath));
            let dimension = Dimension::new(Box::new(loader));

            let (x, z) = coord;

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                let drawer = TopShadeRenderer::new(&pal, height_mode);
                let map = render_region(x, z, dimension, drawer);
                info!("processed r.{}.{}.mca", x.0, z.0);
                Some(map)
            } else {
                None
            }
        })
        .collect();

    info!("{} regions processed", region_maps.len());

    let dx = (x_range.end.0 - x_range.start.0) as usize;
    let dz = (z_range.end.0 - z_range.start.0) as usize;

    let mut img = image::ImageBuffer::new((dx * region_len) as u32, (dz * region_len) as u32);

    for map in region_maps {
        let xrp = map.x.0 - x_range.start.0;
        let zrp = map.z.0 - z_range.start.0;

        for xc in 0..32 {
            for zc in 0..32 {
                let chunk = map.chunk(CCoord(xc), CCoord(zc));
                let xcp = xrp * 32 + xc as isize;
                let zcp = zrp * 32 + zc as isize;

                for z in 0..16 {
                    for x in 0..16 {
                        let pixel = chunk[z * 16 + x];
                        let x = xcp * 16 + x as isize;
                        let z = zcp * 16 + z as isize;
                        img.put_pixel(x as u32, z as u32, image::Rgba(pixel))
                    }
                }
            }
        }
    }

    img.save("map.png").unwrap();
    Ok(())
}

fn tiles(args: &ArgMatches) -> Result<()> {
    let world: PathBuf = args.value_of("world").unwrap().parse().unwrap();
    let dim: &str = args.value_of("dimension").unwrap();
    let out: &str = args.value_of("out").unwrap();
    let height_mode = match args.is_present("calculate-heights") {
        true => HeightMode::Calculate,
        false => HeightMode::Trust,
    };

    let subpath = match dim {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    // don't care if dir already exists.
    std::fs::DirBuilder::new().create(out).unwrap_or_default();

    let loader = RegionFileLoader::new(world.join(subpath));

    let coords = loader.list()?;

    let bounds = match (args.value_of("size"), args.value_of("offset")) {
        (Some(size), Some(offset)) => {
            make_bounds(parse_coord(size).unwrap(), parse_coord(offset).unwrap())
        }
        (None, _) => auto_size(&coords).unwrap(),
        _ => panic!(),
    };

    info!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;

    let pal = get_palette(args.value_of("palette"))?;

    use std::sync::atomic::{AtomicUsize, Ordering};
    let processed_chunks = AtomicUsize::new(0);
    let painted_pixels = AtomicUsize::new(0);

    let regions_processed = coords
        .into_par_iter()
        .map(|coord| {
            let loader = RegionFileLoader::new(world.join(subpath));
            let dimension = Dimension::new(Box::new(loader));

            let (x, z) = coord;

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                let drawer = TopShadeRenderer::new(&pal, height_mode);
                let map = render_region(x, z, dimension, drawer);
                info!("processed r.{}.{}.mca", x.0, z.0);
                Some(map)
            } else {
                None
            }
        })
        .filter_map(|region| region)
        .map(|region| {
            let mut img = image::ImageBuffer::new(region_len as u32, region_len as u32);

            for xc in 0..32 {
                for zc in 0..32 {
                    let heightmap = region.chunk(CCoord(xc), CCoord(zc));
                    let xcp = xc as isize;
                    let zcp = zc as isize;

                    for z in 0..16 {
                        for x in 0..16 {
                            let pixel = heightmap[z * 16 + x];
                            let x = xcp * 16 + x as isize;
                            let z = zcp * 16 + z as isize;
                            img.put_pixel(x as u32, z as u32, image::Rgba(pixel))
                        }
                    }
                }
            }

            img.save(format!("{}/{}.{}.png", out, region.x.0, region.z.0))
                .unwrap();

            ()
        })
        .count();

    info!("{} regions", regions_processed);
    info!("{} chunks", processed_chunks.load(Ordering::SeqCst));
    info!("{} pixels painted", painted_pixels.load(Ordering::SeqCst));
    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let matches = App::new("anvil-fast")
        .subcommand(
            SubCommand::with_name("render")
                .arg(Arg::with_name("world").takes_value(true).required(true))
                .arg(
                    Arg::with_name("size")
                        .long("size")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("offset")
                        .long("offset")
                        .takes_value(true)
                        .required(false)
                        .default_value("0,0"),
                )
                .arg(
                    Arg::with_name("dimension")
                        .long("dimension")
                        .takes_value(true)
                        .required(false)
                        .default_value("overworld"),
                )
                .arg(
                    Arg::with_name("palette")
                        .long("palette")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("jar")
                        .long("jar")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("calculate-heights")
                        .long("calculate-heights")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .subcommand(
            SubCommand::with_name("tiles")
                .arg(Arg::with_name("world").takes_value(true).required(true))
                .arg(
                    Arg::with_name("size")
                        .long("size")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("offset")
                        .long("offset")
                        .takes_value(true)
                        .required(false)
                        .default_value("0,0"),
                )
                .arg(
                    Arg::with_name("dimension")
                        .long("dimension")
                        .takes_value(true)
                        .required(false)
                        .default_value("overworld"),
                )
                .arg(
                    Arg::with_name("out")
                        .long("out")
                        .takes_value(true)
                        .required(false)
                        .default_value("tiles"),
                )
                .arg(
                    Arg::with_name("palette")
                        .long("palette")
                        .takes_value(true)
                        .required(false),
                )
                .arg(
                    Arg::with_name("calculate-heights")
                        .long("calculate-heights")
                        .takes_value(false)
                        .required(false),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("render", Some(args)) => render(args)?,
        ("tiles", Some(args)) => tiles(args)?,
        _ => error!("{}", matches.usage()),
    };

    Ok(())
}
