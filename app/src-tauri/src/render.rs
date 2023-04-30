use std::{fmt::Debug, ops::Deref, sync::Arc};

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
pub fn render_tile(
    window: Window,
    palette: tauri::State<Arc<RenderedPalette>>,
    id: usize,
    tile: TileRequest,
) {
    let palette = palette.deref().clone();
    let heightmap_mode = if tile.heightmap_mode == "calculate" {
        HeightMode::Calculate
    } else {
        HeightMode::Trust
    };

    let subpath = match tile.dimension.as_str() {
        "end" => "DIM1/region",
        "nether" => "DIM-1/region",
        _ => "region",
    };

    let loader = RegionFileLoader::new(format!("{}/{}", tile.world_dir, subpath).into());

    if !loader.has_region(RCoord(tile.rx), RCoord(tile.rz)) {
        window
            .emit(
                "tile_rendered",
                TileResponse::Missing(TileMissing {
                    id,
                    rx: tile.rx,
                    rz: tile.rz,
                    dimension: tile.dimension,
                    world_dir: tile.world_dir,
                }),
            )
            .unwrap();

        return;
    }

    rayon::spawn(move || {
        let render_inner = |tile: TileRequest| -> anyhow::Result<TileResponse> {
            let renderer = TopShadeRenderer::new(palette.deref(), heightmap_mode);
            let map = match render_region(RCoord(tile.rx), RCoord(tile.rz), &loader, renderer) {
                Ok(Some(map)) => map,
                Ok(None) => {
                    return Ok(TileResponse::Missing(TileMissing {
                        id,
                        rx: tile.rx,
                        rz: tile.rz,
                        dimension: tile.dimension,
                        world_dir: tile.world_dir,
                    }));
                }
                Err(e) => return Err(e).context("error loading map"),
            };

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

            Ok(TileResponse::Render(TileRender {
                id,
                rx: tile.rx,
                rz: tile.rz,
                world_dir: tile.world_dir,
                dimension: tile.dimension,
                image_data: buf,
            }))
        };

        match render_inner(tile.clone()) {
            Ok(result) => {
                window.emit("tile_rendered", result).unwrap();
            }
            Err(e) => {
                window
                    .emit(
                        "tile_rendered",
                        TileResponse::Error(TileError {
                            id,
                            rx: tile.rx,
                            rz: tile.rz,
                            dimension: tile.dimension,
                            world_dir: tile.world_dir,
                            message: e.to_string(),
                        }),
                    )
                    .unwrap();
            }
        }
    });
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TileRequest {
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
    heightmap_mode: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "kind")]
pub enum TileResponse {
    Render(TileRender),
    Missing(TileMissing),
    Error(TileError),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TileRender {
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
pub struct TileMissing {
    id: usize,
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TileError {
    id: usize,
    rx: isize,
    rz: isize,
    dimension: String,
    world_dir: String,
    message: String,
}
