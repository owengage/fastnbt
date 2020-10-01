use clap::{App, Arg, ArgMatches, SubCommand};
use fastnbt::anvil::biome::Biome;
use fastnbt::anvil::draw::{parse_region, RegionDrawer, RegionMap, Rgb};
use fastnbt::anvil::Chunk;
use fastnbt::anvil::Region;
use image;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

trait BlockPalette {
    fn pick(&self, block_id: &str, biome: Option<Biome>) -> Rgb;
}

trait IntoMap {
    fn into_map(self) -> RegionMap<Rgb>;
}

struct RegionBlockDrawer<'a, P: BlockPalette + ?Sized> {
    map: RegionMap<Rgb>,
    palette: &'a P,
    processed_chunks: usize,
    painted_pixels: usize,
}

impl<'a, P: BlockPalette + ?Sized> RegionBlockDrawer<'a, P> {
    pub fn new(map: RegionMap<Rgb>, palette: &'a P) -> Self {
        Self {
            map,
            palette,
            processed_chunks: 0,
            painted_pixels: 0,
        }
    }
}

impl<'a, P: BlockPalette + ?Sized> IntoMap for RegionBlockDrawer<'a, P> {
    fn into_map(self) -> RegionMap<Rgb> {
        self.map
    }
}

impl<'a, P: BlockPalette + ?Sized> RegionDrawer for RegionBlockDrawer<'a, P> {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk) {
        let data = self.map.chunk_mut(xc_rel, zc_rel);
        self.processed_chunks += 1;

        for z in 0..16 {
            for x in 0..16 {
                let height = chunk.height_of(x, z).unwrap_or(64);
                let height = if height == 0 { 0 } else { height - 1 }; // -1 because we want the block below the air.
                let biome = chunk.biome_of(x, height, z);
                let material = chunk.id_of(x, height, z);

                // TODO: If material is grass block (and others), we need to colour it based on biome.
                let colour = self.palette.pick(material.unwrap_or(""), biome);

                let pixel = &mut data[x * 16 + z];
                *pixel = colour;
                self.painted_pixels += 1;
            }
        }
    }
}

struct FullPalette {
    blockstates: std::collections::HashMap<String, [u8; 3]>,
    grass: image::RgbImage,
    foliage: image::RgbImage,
}

impl FullPalette {
    fn pick_grass(&self, b: Option<Biome>) -> [u8; 3] {
        b.map(|b| {
            let climate = fastnbt::anvil::biome::climate(b);
            let t = climate.temperature.min(1.).max(0.);
            let r = climate.rainfall.min(1.).max(0.) * t;

            let t = 255 - (t * 255.).ceil() as u32;
            let r = 255 - (r * 255.).ceil() as u32;

            self.grass.get_pixel(t, r).0
        })
        .unwrap_or([0, 0, 0])
    }

    fn pick_foliage(&self, b: Option<Biome>) -> [u8; 3] {
        b.map(|b| {
            let climate = fastnbt::anvil::biome::climate(b);
            let t = climate.temperature.min(1.).max(0.);
            let r = climate.rainfall.min(1.).max(0.) * t;

            let t = 255 - (t * 255.).ceil() as u32;
            let r = 255 - (r * 255.).ceil() as u32;

            self.foliage.get_pixel(t, r).0
        })
        .unwrap_or([0, 0, 0])
    }

    fn pick_water(&self, b: Option<Biome>) -> [u8; 3] {
        b.map(|b| match b {
            Biome::Swamp => [0x61, 0x7B, 0x64],
            Biome::River => [0x3F, 0x76, 0xE4],
            Biome::Ocean => [0x3F, 0x76, 0xE4],
            Biome::LukewarmOcean => [0x45, 0xAD, 0xF2],
            Biome::WarmOcean => [0x43, 0xD5, 0xEE],
            Biome::ColdOcean => [0x3D, 0x57, 0xD6],
            Biome::FrozenRiver => [0x39, 0x38, 0xC9],
            Biome::FrozenOcean => [0x39, 0x38, 0xC9],
            _ => [0x3f, 0x76, 0xe4],
        })
        .unwrap_or([0x3f, 0x76, 0xe4])
    }
}

impl BlockPalette for FullPalette {
    fn pick(&self, block_id: &str, biome: Option<Biome>) -> [u8; 3] {
        // If the block id is something like grass, we should pull the colour from the colour map.
        match block_id {
            "minecraft:grass_block" => return self.pick_grass(biome),
            "minecraft:oak_leaves" => return self.pick_foliage(biome),
            "minecraft:jungle_leaves" => return self.pick_foliage(biome),
            "minecraft:acacia_leaves" => return self.pick_foliage(biome),
            "minecraft:dark_oak_leaves" => return self.pick_foliage(biome),
            "minecraft:water" => return self.pick_water(biome),

            // Specific colours defined.
            "minecraft:birch_leaves" => return [0x80, 0xa7, 0x55],
            "minecraft:spruce_leaves" => return [0x61, 0x99, 0x61],

            // FIXME: How do we colour snow?
            "minecraft:snow" => return [250, 250, 250],

            // FIXME: We should probably render the floor under the vine. We cheat here and render it like
            // foliage.
            "minecraft:vine" => return self.pick_foliage(biome),

            // Occurs a lot for the end, as layer 0 will be air a lot.
            // Rendering it black makes sense in the end, but might look weird if it ends up elsewhere.
            "minecraft:air" => return [0, 0, 0],

            _ => {}
        }

        let col = self.blockstates.get(block_id);
        match col {
            Some(c) => *c,
            None => {
                // println!("could not draw {}", block_id);
                [255, 0, 255]
            }
        }
    }
}

struct RegionBiomeDrawer<'a> {
    map: &'a mut RegionMap<Rgb>,
}

impl<'a> RegionDrawer for RegionBiomeDrawer<'a> {
    fn draw(&mut self, xc_rel: usize, zc_rel: usize, chunk: &mut Chunk) {
        let data = (*self.map).chunk_mut(xc_rel, zc_rel);

        for z in 0..16 {
            for x in 0..16 {
                let y = chunk.height_of(x, z).unwrap_or(64);
                let biome = chunk.biome_of(x, y, z).unwrap_or(Biome::TheVoid);

                // TODO:  If material is grass block (and others), we need to colour it based on biome.
                let colour = match biome {
                    Biome::Ocean => [0, 0, 200],
                    Biome::DeepOcean => [0, 0, 150],
                    Biome::ColdOcean
                    | Biome::DeepColdOcean
                    | Biome::DeepFrozenOcean
                    | Biome::DeepLukewarmOcean
                    | Biome::DeepWarmOcean
                    | Biome::FrozenOcean
                    | Biome::LukewarmOcean
                    | Biome::WarmOcean => [0, 0, 255],
                    Biome::River | Biome::FrozenRiver => [0, 0, 255],
                    Biome::Beach => [100, 50, 50],

                    b => {
                        let b: i32 = b.into();
                        [b as u8, b as u8, b as u8]
                    }
                };

                let pixel = &mut data[x * 16 + z];
                *pixel = colour;
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
    let mut parts = filename.split('.').rev().skip(1);
    let z = parts.next()?.parse::<isize>().ok()?;
    let x = parts.next()?.parse::<isize>().ok()?;
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
        None => panic!("no palette"),
    };

    let f = std::fs::File::open(path)?;
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

                grass =
                    Ok(image::load(std::io::Cursor::new(buf), image::ImageFormat::Png)?.into_rgb());
            }
            "foliage-colourmap.png" => {
                use std::io::Read;
                let mut buf = vec![];
                file.read_to_end(&mut buf)?;

                foliage =
                    Ok(image::load(std::io::Cursor::new(buf), image::ImageFormat::Png)?.into_rgb());
            }
            "blockstates.json" => {
                let json: std::collections::HashMap<String, [u8; 3]> =
                    serde_json::from_reader(file)?;
                blockstates = Ok(json);
            }
            _ => {}
        }
    }

    let p = FullPalette {
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

    print!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;
    let dx = x_range.len();
    let dz = z_range.len();

    let pal: std::sync::Arc<dyn BlockPalette + Send + Sync> =
        get_palette(args.value_of("palette"))?.into();

    use std::sync::atomic::{AtomicUsize, Ordering};
    let processed_chunks = AtomicUsize::new(0);
    let painted_pixels = AtomicUsize::new(0);

    let region_maps: Vec<Option<RegionMap<Rgb>>> = paths
        .into_par_iter()
        .map(|path| {
            let (x, z) = coords_from_region(&path).unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                println!("parsing region x: {}, z: {}", x, z);
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let map = RegionMap::new(x, z, [0, 0, 0]);
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

    println!("{} regions", region_maps.len());
    println!("{} chunks", processed_chunks.load(Ordering::SeqCst));
    println!("{} pixels painted", painted_pixels.load(Ordering::SeqCst));

    println!("1 map.png");
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

fn biomes(args: &ArgMatches) -> Result<()> {
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

    print!("Bounds: {:?}", bounds);

    let x_range = bounds.xmin..bounds.xmax;
    let z_range = bounds.zmin..bounds.zmax;

    let region_len: usize = 32 * 16;
    let dx = x_range.len();
    let dz = z_range.len();

    let region_maps: Vec<Option<RegionMap<Rgb>>> = paths
        .into_par_iter()
        .map(|path| {
            let (x, z) = coords_from_region(&path).unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                println!("parsing region x: {}, z: {}", x, z);
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let mut map = RegionMap::new(x, z, [0, 0, 0]);
                let mut drawer = RegionBiomeDrawer { map: &mut map };
                parse_region(region, &mut drawer).unwrap_or_default(); // TODO handle some of the errors here

                Some(map)
            } else {
                None
            }
        })
        .collect();

    println!("writing biome.png");
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

    img.save("biome.png").unwrap();
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
        .subcommand(
            SubCommand::with_name("biomes")
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
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("render", Some(args)) => render(args)?,
        ("biomes", Some(args)) => biomes(args)?,
        _ => println!("{}", matches.usage()),
    };

    Ok(())
}
