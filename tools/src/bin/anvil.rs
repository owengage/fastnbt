use clap::{App, Arg, ArgMatches, SubCommand};
use env_logger::Env;
use fastanvil::Region;
use fastanvil::RenderedPalette;
use fastanvil::{parse_region, RegionBlockDrawer, RegionMap, Rgba};
use fastanvil::{IntoMap, Palette};
use fastnbt_tools::make_palette;
use flate2::read::GzDecoder;
use image;
use log::{error, info};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn parse_coord(coord: &str) -> Option<(isize, isize)> {
    let mut s = coord.split(",");
    let x: isize = s.next()?.parse().ok()?;
    let z: isize = s.next()?.parse().ok()?;
    Some((x, z))
}

/// Get all the paths to region files in a 'region' directory like 'region', 'DIM1' and 'DIM-1'.
fn region_paths(in_path: &Path) -> Result<Vec<PathBuf>> {
    let paths = std::fs::read_dir(in_path)?;

    let paths = paths
        .into_iter()
        .filter_map(|path| path.ok())
        .map(|path| path.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let ext = path.extension();
            ext.is_some() && ext.unwrap() == "mca"
        })
        .filter(|path| fs::metadata(path).unwrap().len() > 0)
        .collect();

    Ok(paths)
}

fn coords_from_region(region: &Path) -> Option<(isize, isize)> {
    let filename = region.file_name()?.to_str()?;
    let mut parts = filename.split('.').skip(1);
    let x = parts.next()?.parse::<isize>().ok()?;
    let z = parts.next()?.parse::<isize>().ok()?;
    Some((x, z))
}

fn auto_size(paths: &Vec<PathBuf>) -> Option<Rectangle> {
    if paths.len() == 0 {
        return None;
    }

    let mut bounds = Rectangle {
        xmin: isize::MAX,
        zmin: isize::MAX,
        xmax: isize::MIN,
        zmax: isize::MIN,
    };

    for path in paths {
        let coord = coords_from_region(path)?;
        bounds.xmin = std::cmp::min(bounds.xmin, coord.0);
        bounds.xmax = std::cmp::max(bounds.xmax, coord.0);
        bounds.zmin = std::cmp::min(bounds.zmin, coord.1);
        bounds.zmax = std::cmp::max(bounds.zmax, coord.1);
    }

    Some(bounds)
}

fn make_bounds(size: (isize, isize), off: (isize, isize)) -> Rectangle {
    Rectangle {
        xmin: off.0 - (size.0 + 0) / 2, // size + 1 makes sure that a size of 1,1
        xmax: off.0 + (size.0 + 1) / 2, // produces bounds of size 1,1 rather than
        zmin: off.1 - (size.1 + 0) / 2, // the 0,0 you would get without it.
        zmax: off.1 + (size.1 + 1) / 2,
    }
}

#[derive(Debug)]
struct Rectangle {
    xmin: isize,
    xmax: isize,
    zmin: isize,
    zmax: isize,
}

fn get_palette(path: Option<&str>) -> Result<Box<dyn Palette + Sync + Send>> {
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

    Ok(Box::new(p))
}

fn render(args: &ArgMatches) -> Result<()> {
    let world: PathBuf = args.value_of("world").unwrap().parse().unwrap();
    let dim: &str = args.value_of("dimension").unwrap();

    let subpath = match dim {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    let paths = region_paths(&world.join(subpath))
        .or(Err(format!("no region files found for {} dimension", dim)))?;

    let bounds = match (args.value_of("size"), args.value_of("offset")) {
        (Some(size), Some(offset)) => {
            make_bounds(parse_coord(size).unwrap(), parse_coord(offset).unwrap())
        }
        (None, _) => auto_size(&paths).unwrap(),
        _ => panic!(),
    };

    info!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;
    let dx = x_range.len();
    let dz = z_range.len();

    let pal_path = args.value_of("palette");
    let pal: std::sync::Arc<dyn Palette + Send + Sync> = match pal_path {
        Some(p) => get_palette(Some(p))?.into(),
        None => {
            let jar = args
                .value_of("jar")
                .ok_or("must provide either --palette or --jar")?;
            make_palette(Path::new(jar))?;
            get_palette(Some("palette.tar.gz"))?.into()
        }
    };

    use std::sync::atomic::{AtomicUsize, Ordering};
    let processed_chunks = AtomicUsize::new(0);
    let painted_pixels = AtomicUsize::new(0);

    let region_maps: Vec<Option<RegionMap<Rgba>>> = paths
        .into_par_iter()
        .map(|path| {
            let (x, z) = coords_from_region(&path).unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                info!("parsing region x: {}, z: {}", x, z);
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let map = RegionMap::new(x, z, [0, 0, 0, 0]);
                let mut drawer = RegionBlockDrawer::new(map, &*pal);
                parse_region(region, &mut drawer).unwrap_or_default(); // TODO handle some of the errors here

                processed_chunks.fetch_add(drawer.processed_chunks, Ordering::SeqCst);
                painted_pixels.fetch_add(drawer.painted_pixels, Ordering::SeqCst);

                Some(drawer.into_map())
            } else {
                None
            }
        })
        .collect();

    info!("{} regions", region_maps.len());
    info!("{} chunks", processed_chunks.load(Ordering::SeqCst));
    info!("{} pixels painted", painted_pixels.load(Ordering::SeqCst));

    info!("1 map.png");
    let mut img = image::ImageBuffer::new((dx * region_len) as u32, (dz * region_len) as u32);

    for region_map in region_maps {
        if let Some(map) = region_map {
            let xrp = map.x_region - x_range.start;
            let zrp = map.z_region - z_range.start;

            for xc in 0..32 {
                for zc in 0..32 {
                    let chunk = map.chunk(xc, zc);
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
    }

    img.save("map.png").unwrap();
    Ok(())
}

fn tiles(args: &ArgMatches) -> Result<()> {
    let world: PathBuf = args.value_of("world").unwrap().parse().unwrap();
    let dim: &str = args.value_of("dimension").unwrap();

    let subpath = match dim {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    let paths = region_paths(&world.join(subpath))
        .or(Err(format!("no region files found for {} dimension", dim)))?;

    let bounds = match (args.value_of("size"), args.value_of("offset")) {
        (Some(size), Some(offset)) => {
            make_bounds(parse_coord(size).unwrap(), parse_coord(offset).unwrap())
        }
        (None, _) => auto_size(&paths).unwrap(),
        _ => panic!(),
    };

    info!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;

    let pal: std::sync::Arc<dyn Palette + Send + Sync> =
        get_palette(args.value_of("palette"))?.into();

    use std::sync::atomic::{AtomicUsize, Ordering};
    let processed_chunks = AtomicUsize::new(0);
    let painted_pixels = AtomicUsize::new(0);

    paths
        .into_par_iter()
        .map(|path| {
            let (x, z) = coords_from_region(&path).unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                info!("parsing region x: {}, z: {}", x, z);
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let map = RegionMap::new(x, z, [0, 0, 0, 0]);
                let mut drawer = RegionBlockDrawer::new(map, &*pal);
                parse_region(region, &mut drawer).unwrap_or_default(); // TODO handle some of the errors here

                processed_chunks.fetch_add(drawer.processed_chunks, Ordering::SeqCst);
                painted_pixels.fetch_add(drawer.painted_pixels, Ordering::SeqCst);

                Some(drawer.into_map())
            } else {
                None
            }
        })
        .filter_map(|region| region)
        .for_each(|region| {
            let mut img = image::ImageBuffer::new(region_len as u32, region_len as u32);

            for xc in 0..32 {
                for zc in 0..32 {
                    let heightmap = region.chunk(xc, zc);
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

            img.save(format!("tiles/{}.{}.png", region.x_region, region.z_region))
                .unwrap();
        });

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
                    Arg::with_name("palette")
                        .long("palette")
                        .takes_value(true)
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
