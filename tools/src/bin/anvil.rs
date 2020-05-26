use clap::{App, Arg, ArgMatches, SubCommand};
use fastnbt::anvil::draw::{parse_region, BasicPalette, RegionBlockDrawer, RegionMap};
use fastnbt::anvil::Region;
use image;
use rayon::prelude::*;
use std::path::PathBuf;

fn render(args: &ArgMatches) {
    let minx: isize = args.value_of("min-x").unwrap().parse().unwrap();
    let minz: isize = args.value_of("min-z").unwrap().parse().unwrap();
    let maxx: isize = args.value_of("max-x").unwrap().parse().unwrap();
    let maxz: isize = args.value_of("max-z").unwrap().parse().unwrap();
    let world: PathBuf = args.value_of("world").unwrap().parse().unwrap();
    let dim: &str = args.value_of("dimension").unwrap();

    let x_range = minx..maxx;
    let z_range = minz..maxz;

    let region_len: usize = 32 * 16;
    let dx = x_range.len();
    let dz = z_range.len();

    let subpath = match dim {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    let paths = std::fs::read_dir(world.join(subpath)).unwrap();

    let paths: Vec<_> = paths
        .into_iter()
        .filter_map(|path| path.ok())
        .map(|path| path.path())
        .filter(|path| path.is_file())
        .collect();

    let region_maps: Vec<_> = paths
        .into_par_iter()
        .map(|path| {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let mut parts = filename.split('.').skip(1);
            let x = parts.next().unwrap().parse::<isize>().unwrap();
            let z = parts.next().unwrap().parse::<isize>().unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                println!("parsing region x: {}, z: {}", x, z);
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let mut map = RegionMap::new(x, z, image::Rgb::<u8>([0, 0, 0]));
                let palette = BasicPalette {};
                let mut drawer = RegionBlockDrawer::new(&mut map, &palette);
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
                            img.put_pixel(x as u32, z as u32, pixel)
                        }
                    }
                }
            }
        }
    }

    img.save("map.png").unwrap();
}

fn main() {
    let matches = App::new("anvil-fast")
        .subcommand(
            SubCommand::with_name("render")
                .arg(Arg::with_name("world").takes_value(true).required(true))
                .arg(
                    Arg::with_name("max-z")
                        .long("max-z")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("max-x")
                        .long("max-x")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("min-z")
                        .long("min-z")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("min-x")
                        .long("min-x")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("dimension")
                        .long("dimension")
                        .takes_value(true)
                        .required(false)
                        .default_value("overworld"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("render", Some(args)) => render(args),
        _ => println!("{}", matches.usage()),
    };
}
