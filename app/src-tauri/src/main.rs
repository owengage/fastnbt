// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod palette;

use std::{ops::Deref, sync::Arc};

use anyhow::Context;
use base64_serde::base64_serde_type;
use fastanvil::{
    render_region, CCoord, HeightMode, RCoord, RegionFileLoader, RenderedPalette, TopShadeRenderer,
};
use image::Pixel;
use serde::{Deserialize, Serialize};
use tauri::Window;

base64_serde_type!(Base64Standard, base64::engine::general_purpose::STANDARD);

#[tauri::command]
fn render_tile(
    window: Window,
    palette: tauri::State<Arc<RenderedPalette>>,
    id: usize,
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
    heightmap_mode: String,
) {
    let palette = palette.deref().clone();

    rayon::spawn(move || {
        let tile = TileRequest {
            rx,
            rz,
            dimension,
            world_dir,
            heightmap_mode,
        };
        let render_inner = |tile: TileRequest| -> anyhow::Result<TileRender> {
            let heightmap_mode = if tile.heightmap_mode == "calculate" {
                HeightMode::Calculate
            } else {
                HeightMode::Trust
            };
            let renderer = TopShadeRenderer::new(palette.deref(), heightmap_mode);
            let loader = RegionFileLoader::new(format!("{}/{}", tile.world_dir, "region").into());

            let map =
                render_region(RCoord(rx), RCoord(rz), &loader, renderer)?.context("no map")?;

            let region_len: usize = 32 * 16;

            let mut img = image::ImageBuffer::new(region_len as u32, region_len as u32);

            for xc in 0..32 {
                for zc in 0..32 {
                    let chunk = map.chunk(CCoord(xc), CCoord(zc));
                    let xcp = xc;
                    let zcp = zc;

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

            let mut buf = vec![];
            let enc = image::png::PngEncoder::new(&mut buf);

            enc.encode(
                img.as_raw(),
                img.width(),
                img.height(),
                image::Rgba::<u8>::COLOR_TYPE,
            )
            .unwrap();

            Ok(TileRender {
                id,
                rx: tile.rx,
                rz: tile.rz,
                world_dir: tile.world_dir,
                dimension: tile.dimension,
                image_data: buf,
            })
        };

        match render_inner(tile.clone()) {
            Ok(rendered) => {
                let _emit = window.emit("tile_rendered", TileResponse::Render(rendered));
            }
            Err(e) => {
                let _emit = window.emit(
                    "tile-rendered",
                    TileResponse::Error(TileError {
                        id,
                        rx: tile.rx,
                        rz: tile.rz,
                        dimension: tile.dimension,
                        world_dir: tile.world_dir,
                        message: e.to_string(),
                    }),
                );
            }
        }
    });
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TileRequest {
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
    heightmap_mode: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
enum TileResponse {
    Render(TileRender),
    Error(TileError),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TileRender {
    id: usize,
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
    #[serde(with = "Base64Standard")]
    image_data: Vec<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TileError {
    id: usize,
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
    message: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let palette = Arc::new(palette::get_palette()?);

    tauri::Builder::default()
        .manage(palette)
        .invoke_handler(tauri::generate_handler![render_tile])
        .run(tauri::generate_context!())
        .context("error while running tauri application")
}
