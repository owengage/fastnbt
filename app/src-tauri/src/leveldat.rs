use std::{fs::File, io::BufReader, path::PathBuf};

use crate::error::{JsError, JsResult};
use fastnbt::Value;
use flate2::bufread::GzDecoder;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldInfo {
    dir: String,
    level: Value,
}

#[tauri::command]
pub fn world_list(dir: String) -> JsResult<Vec<WorldInfo>> {
    let entries = std::fs::read_dir(dir).map_err(JsError::new)?;

    Ok(entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .flat_map(|entry| -> anyhow::Result<WorldInfo> {
            let path = entry.path().canonicalize()?;
            let level = BufReader::new(File::open(path.join("level.dat"))?);
            let dec = GzDecoder::new(level);

            let level: Value = fastnbt::from_reader(dec)?;

            Ok(WorldInfo {
                dir: path.to_string_lossy().into_owned(),
                level,
            })
        })
        .collect())
}

#[tauri::command]
pub fn world_info(dir: String) -> JsResult<WorldInfo> {
    let dir = PathBuf::from(dir).canonicalize().map_err(JsError::new)?;

    if dir.is_dir() {
        let level = BufReader::new(File::open(dir.join("level.dat")).map_err(JsError::new)?);
        let dec = GzDecoder::new(level);
        let level: Value = fastnbt::from_reader(dec).map_err(JsError::new)?;

        Ok(WorldInfo {
            dir: dir.to_string_lossy().into_owned(),
            level,
        })
    } else {
        Err(JsError::new("world path was not a directory"))
    }
}
