#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod cache;
mod selector;
mod ui;
mod win32;

mod utils;

#[tokio::main]
async fn main() {
    ui::ui_init().await;
}
