use clap::{App, Arg, ArgMatches, SubCommand};
use fastnbt::anvil::draw::{
    parse_region, BasicPalette, BlockPalette, RegionBlockDrawer, RegionMap,
};
use fastnbt::anvil::Region;
use image;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct FullPalette(std::collections::HashMap<String, [u8; 3]>);

impl BlockPalette for FullPalette {
    fn pick(&self, block_id: &str) -> [u8; 3] {
        let col = self.0.get(block_id);
        match col {
            Some(c) => *c,
            None => {
                println!("{}", block_id);
                [255, 0, 255]
            }
        }
    }
}

fn parse_coord(coord: &str) -> Option<(isize, isize)> {
    let mut s = coord.split(",");
    let x: isize = s.next()?.parse().ok()?;
    let z: isize = s.next()?.parse().ok()?;
    Some((x, z))
}

fn region_paths(in_path: &Path) -> Vec<PathBuf> {
    let paths = std::fs::read_dir(in_path).unwrap();

    paths
        .into_iter()
        .filter_map(|path| path.ok())
        .map(|path| path.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            let ext = path.extension();
            ext.is_some() && ext.unwrap() == "mca"
        })
        .collect()
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
        xmin: off.0 - size.0 / 2,
        xmax: off.0 + size.0 / 2,
        zmin: off.1 - size.1 / 2,
        zmax: off.1 + size.1 / 2,
    }
}

#[derive(Debug)]
struct Rectangle {
    xmin: isize,
    xmax: isize,
    zmin: isize,
    zmax: isize,
}

fn get_palette(path: Option<&str>) -> Result<Box<dyn BlockPalette + Sync + Send>> {
    let path = match path {
        Some(path) => Path::new(path),
        None => return Ok(Box::new(BasicPalette {})),
    };

    let f = std::fs::File::open(path)?;

    let mut json: std::collections::HashMap<String, [u8; 3]> = serde_json::from_reader(f)?;

    Ok(Box::new(FullPalette(json)))
}

fn render(args: &ArgMatches) -> Result<()> {
    let world: PathBuf = args.value_of("world").unwrap().parse().unwrap();
    let dim: &str = args.value_of("dimension").unwrap();

    let subpath = match dim {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    let paths = region_paths(&world.join(subpath));

    let bounds = match (args.value_of("size"), args.value_of("offset")) {
        (Some(size), Some(offset)) => {
            make_bounds(parse_coord(size).unwrap(), parse_coord(offset).unwrap())
        }
        (None, _) => auto_size(&paths).unwrap(),
        _ => panic!(),
    };

    print!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;
    let dx = x_range.len();
    let dz = z_range.len();

    let pal: std::sync::Arc<dyn BlockPalette + Send + Sync> =
        get_palette(args.value_of("palette"))?.into();

    let region_maps: Vec<_> = paths
        .into_par_iter()
        .map(|path| {
            let (x, z) = coords_from_region(&path).unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                println!("parsing region x: {}, z: {}", x, z);
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let mut map = RegionMap::new(x, z, [0, 0, 0]);
                let mut drawer = RegionBlockDrawer::new(&mut map, &*pal);
                parse_region(region, &mut drawer).unwrap_or_default(); // TODO handle some of the errors here

                Some(map)
            } else {
                None
            }
        })
        .collect();

    println!("writing map.png");
    let mut img = image::ImageBuffer::new((dx * region_len) as u32, (dz * region_len) as u32);

    for region_map in region_maps {
        if let Some(map) = region_map {
            let xrp = map.x_region - x_range.start;
            let zrp = map.z_region - z_range.start;

            for xc in 0..32 {
                for zc in 0..32 {
                    let heightmap = map.chunk(xc, zc);
                    let xcp = xrp * 32 + xc as isize;
                    let zcp = zrp * 32 + zc as isize;

                    for z in 0..16 {
                        for x in 0..16 {
                            let pixel = heightmap[z * 16 + x];
                            let x = xcp * 16 + x as isize;
                            let z = zcp * 16 + z as isize;
                            img.put_pixel(x as u32, z as u32, image::Rgb(pixel))
                        }
                    }
                }
            }
        }
    }

    img.save("map.png").unwrap();
    Ok(())
}

fn main() -> Result<()> {
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
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("render", Some(args)) => render(args)?,
        _ => println!("{}", matches.usage()),
    };

    Ok(())
}
