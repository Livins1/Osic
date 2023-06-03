// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod gallery;
mod utils;

use gallery::GalleryState;

fn main() {


    tauri::Builder::default()
        .manage(GalleryState::new())
        .invoke_handler(tauri::generate_handler![
            gallery::add_folder,
            gallery::get_folders,
            gallery::preview
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
