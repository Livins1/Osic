use std::sync::Arc;
use std::sync::Mutex;

use tauri::{command, State, Window};

#[derive(Debug)]
pub struct Rect {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

impl Rect {
    pub fn from_win32(rect: windows::Win32::Foundation::RECT) -> Self {
        Self {
            left: rect.left,
            right: rect.right,
            top: rect.top,
            bottom: rect.bottom,
        }
    }
}

#[derive(Debug)]
pub struct Monitor {
    pub title: String,
    pub id: String,
    pub gdi_name: String,
    pub rect: Rect,
}
impl Monitor {
    pub fn from_win32(
        title: String,
        id: String,
        gdi_name: String,
        rect: windows::Win32::Foundation::RECT,
    ) -> Self {
        Self {
            title,
            id,
            gdi_name,
            rect: Rect::from_win32(rect),
        }
    }
}
pub struct MonitorState(Arc<Mutex<MonitorHandle>>);
pub type MonitorsArg<'a> = State<'a, MonitorState>;

impl MonitorState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(MonitorHandle::new())))
    }
}

pub struct MonitorHandle {
    monitors: Vec<Monitor>,
    win32_api: crate::win32::Win32API,
}

impl MonitorHandle {
    fn new() -> Self {
        let win32_api = crate::win32::Win32API::new();
        match win32_api.get_monitors() {
            Ok(monitors) => Self {
                monitors,
                win32_api,
            },
            Err(_) => Self {
                monitors: Vec::new(),
                win32_api,
            },
        }
    }
    pub fn preview_as_wallpaper(&self, img_path: String) {
        if let Some(monitor) = self.monitors.first() {
            let _ = self.win32_api.set_wallpaper(&monitor.id, &img_path);
        };
    }
}

#[command]
pub fn preview_as_wallpaper(monitor: MonitorsArg<'_>, img_path: String) -> Result<(), String> {
    let handle = monitor.0.lock().unwrap();
    handle.preview_as_wallpaper(img_path);
    Ok(())
}
