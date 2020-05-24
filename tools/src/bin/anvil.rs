use fastnbt::anvil::draw::{parse_region, DrawResult, RegionBlockDrawer, RegionMap};
use fastnbt::anvil::Region;
use image;
use rayon::prelude::*;
use std::path::Path;

fn main() -> DrawResult<()> {
    let args: Vec<String> = std::env::args().collect();
    let paths = &args[1..];

    let x_range = -3isize..3;
    let z_range = -3isize..3;

    let region_len: usize = 32 * 16;
    let dx = x_range.len();
    let dz = z_range.len();

    let mut img = image::ImageBuffer::new((dx * region_len) as u32, (dz * region_len) as u32);

    let region_maps: Vec<_> = paths
        .par_iter()
        .map(|path| {
            let path = Path::new(path);

            let filename = path.file_name().unwrap().to_str().unwrap();
            let mut parts = filename.split('.').skip(1);
            let x = parts.next().unwrap().parse::<isize>().unwrap();
            let z = parts.next().unwrap().parse::<isize>().unwrap();

            if x < x_range.end && x >= x_range.start && z < z_range.end && z >= z_range.start {
                let file = std::fs::File::open(path).ok()?;
                let region = Region::new(file);

                let mut map = RegionMap::new(x, z, image::Rgb::<u8>([0, 0, 0]));
                let mut drawer = RegionBlockDrawer::new(&mut map);
                parse_region(region, &mut drawer).unwrap_or_default(); // TODO handle some of the errors here

                Some(map)
            } else {
                None
            }
        })
        .collect();

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

    img.save("test.png").unwrap();

    Ok(())
}
