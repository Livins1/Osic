#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ui;
mod cache;
mod win32;
mod selector;


mod utils;

fn main() {
    ui::ui_init();
}