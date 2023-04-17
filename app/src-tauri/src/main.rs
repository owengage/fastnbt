// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    let entries: String = std::fs::read_dir(".")
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            entry.file_name().to_str().unwrap().to_owned()
        })
        .collect();

    format!("Files: {}", entries)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
