use std::io::Cursor;

use fastanvil::RegionMap;
use fastanvil::{parse_region, Region, RegionBlockDrawer, RenderedPalette};
use palette::get_palette;
use wasm_bindgen::prelude::*;

mod biome;
mod palette;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct TileRenderer {
    pal: RenderedPalette,
}

#[wasm_bindgen]
impl TileRenderer {
    pub fn new() -> Self {
        Self {
            pal: get_palette().unwrap(),
        }
    }

    pub fn render(&self, region: &[u8]) -> Vec<u8> {
        let cursor = Cursor::new(region);
        let region = Region::new(cursor);

        let map = RegionMap::new(0, 0, [0, 0, 0, 0]);
        let mut drawer = RegionBlockDrawer::new(map, &self.pal);

        parse_region(region, &mut drawer).unwrap_or_default(); // TODO handle some of the errors here

        let region = drawer.map;
        let region_len: usize = 32 * 16;

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

        img.into_raw()
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(start)]
pub fn force_init() {
    set_panic_hook();
}
