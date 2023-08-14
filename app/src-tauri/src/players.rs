use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::error::{JsError, JsResult};

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    id: String,
}

#[tauri::command]
pub fn player_list(dir: String) -> JsResult<Vec<Player>> {
    let dir = Path::new(&dir);
    let playerdata = dir.join("playerdata");

    let ids: Vec<Player> = fs::read_dir(playerdata)
        .map_err(JsError::new)?
        .flatten()
        .flat_map(|playerpath| {
            let id = playerpath.file_name();
            let id = id.into_string();
            match id {
                // Sometimes contains files with `.dat_old` extension.
                Ok(id) if id.ends_with(".dat") => Some(id),
                _ => None,
            }
        })
        .map(|id| Player { id })
        .collect();

    Ok(ids)
}
