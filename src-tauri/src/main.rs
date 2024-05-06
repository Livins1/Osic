// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{Event, Manager, SystemTray, SystemTrayEvent, WindowEvent};

mod core;
mod tray;

use core::display;
use core::display::DisplayState;

fn main() {
    let mut builder = tauri::Builder::default();

    builder = builder
        .system_tray(SystemTray::new().with_menu(tray::build_menu()))
        .on_system_tray_event(move |app, event| match event {
            SystemTrayEvent::DoubleClick {
                tray_id: _,
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { tray_id: _, id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }

                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        });

    builder
        .manage(DisplayState::new())
        .invoke_handler(tauri::generate_handler![core::display::display_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
