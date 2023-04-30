// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod render;

use std::{
    io::{Cursor, Read},
    path::PathBuf,
    sync::Arc,
};

use anyhow::Context;
use fastnbt::Value;
use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};

use crate::render::render_tile;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorldInfo {
    dir: String,
    level: Value,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JsError {
    message: String,
}
impl JsError {
    pub fn new<E: ToString>(e: E) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}

#[tauri::command]
fn world_list(dir: String) -> Result<Vec<WorldInfo>, JsError> {
    let entries = std::fs::read_dir(dir).map_err(JsError::new)?;

    Ok(entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .flat_map(|entry| -> anyhow::Result<WorldInfo> {
            let path = entry.path().canonicalize()?;
            let level = std::fs::read(path.join("level.dat"))?;
            let mut dec = GzDecoder::new(Cursor::new(&level));
            let mut level = vec![];
            dec.read_to_end(&mut level)?;

            let level: Value = fastnbt::from_bytes(&level)?;

            Ok(WorldInfo {
                dir: path.to_string_lossy().into_owned(),
                level,
            })
        })
        .collect())
}

#[tauri::command]
fn world_info(dir: String) -> Result<WorldInfo, JsError> {
    let dir = PathBuf::from(dir).canonicalize().map_err(JsError::new)?;

    if dir.is_dir() {
        let level = std::fs::read(dir.join("level.dat")).map_err(JsError::new)?;
        let mut dec = GzDecoder::new(Cursor::new(&level));
        let mut level = vec![];
        dec.read_to_end(&mut level).map_err(JsError::new)?;

        let level: Value = fastnbt::from_bytes(&level).map_err(JsError::new)?;

        Ok(WorldInfo {
            dir: dir.to_string_lossy().into_owned(),
            level,
        })
    } else {
        Err(JsError {
            message: "world path was not a directory".to_string(),
        })
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let data = include_bytes!("../../../palette.tar.gz");
    let palette = Arc::new(fastanvil::load_rendered_palette(Cursor::new(data))?);

    tauri::Builder::default()
        .manage(palette)
        .invoke_handler(tauri::generate_handler![
            render_tile,
            world_list,
            world_info
        ])
        .run(tauri::generate_context!())
        .context("error while running tauri application")
}
