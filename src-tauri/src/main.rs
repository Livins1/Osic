// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod gallery;
mod utils;
mod monitor;
mod win32;
    

    

use gallery::GalleryState;

fn main() {

    win32::test_monitor_function();

    let app_cache = cache::AppCache::new("Osic.cache");

    tauri::Builder::default()
        .manage(GalleryState::new(app_cache))
        .invoke_handler(tauri::generate_handler![
            gallery::add_folder,
            gallery::get_folders,
            gallery::rescan_folder,
            gallery::remove_folder,
            gallery::preview,
            gallery::explorer_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
