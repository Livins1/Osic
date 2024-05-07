use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::{collections::VecDeque, path::PathBuf};
use tauri::{command, InvokePayload, PageLoadPayload, State, Window};

use super::{cache::OsicRecentImage, selector::OsicSlideSelector};

use super::win32::Win32API;
const MODES: &'static [&'static str] = &["Picture", "SlidShow"];
const FITS: &'static [&'static str] = &["Fill", "Fit", "Stretch", "Tile", "Center", "Span"];
const INTERVAL: &'static [&'static str] =
    &["1 minute", "10 minutes", "30 minutes", "1 hour", "6 hour"];

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Modes {
    Picture,
    SlidShow,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Fits {
    Fill = 4,
    Fit = 3,
    Stretch = 2,
    Tile = 1,
    Center = 0,
    Span = 5,
}
impl Fits {
    fn find(mode: &str) -> Fits {
        match mode {
            "Fill" => Fits::Fill,
            "Fit" => Fits::Fit,
            "Stretch" => Fits::Stretch,
            "Tile" => Fits::Tile,
            "Center" => Fits::Center,
            "Span" => Fits::Span,
            _ => Fits::Fill,
        }
    }
}

impl Modes {
    fn find(mode: &str) -> Modes {
        match mode {
            "Picture" => Modes::Picture,
            "SlidShow" => Modes::SlidShow,
            _ => Modes::Picture,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Interval {
    OneMinute,
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    SixHours,
}

impl Interval {
    fn find(i: &str) -> Interval {
        match i {
            "1 minute" => Interval::OneMinute,
            "10 minutes" => Interval::TenMinutes,
            "30 minutes" => Interval::ThirtyMinutes,
            "1 hour" => Interval::OneHour,
            "6 hour" => Interval::SixHours,
            _ => Interval::TenMinutes,
        }
    }

    fn seconds(i: &Interval) -> u64 {
        match i {
            Interval::OneMinute => 60,
            Interval::TenMinutes => 600,
            Interval::ThirtyMinutes => 60 * 30,
            Interval::OneHour => 60 * 60,
            Interval::SixHours => 60 * 60 * 6,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1 minute",
            Interval::TenMinutes => "10 minutes",
            Interval::ThirtyMinutes => "30 minutes",
            Interval::OneHour => "1 hour",
            Interval::SixHours => "6 hours",
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Monitor {
    pub name: String,
    pub device_id: String,
    pub letf: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorWrapper {
    meta: Monitor,
    pub device_id: String,
    pub album_path: Option<PathBuf>,
    pub image: Option<PathBuf>,
    pub image_history: VecDeque<OsicRecentImage>,
    pub fit: Fits,
    pub slide_interval: Interval,
    pub slide_time: u64,
    pub mode: Modes,
    pub selector: OsicSlideSelector,
}
impl MonitorWrapper {
    fn new(monitor: Monitor) -> Self {
        let ratio_value: f32 = monitor.width as f32 / monitor.height as f32;
        Self {
            device_id: monitor.device_id.clone(),
            meta: monitor,
            mode: Modes::Picture,
            image: None,
            image_history: VecDeque::default(),
            fit: Fits::Fill,
            album_path: None,
            slide_interval: Interval::TenMinutes,
            slide_time: 0,
            selector: OsicSlideSelector::new(ratio_value),
        }
    }
}
pub struct DisplayState(Arc<Mutex<DisplayHandle>>);
pub type DisplayArg<'a> = State<'a, DisplayState>;

impl DisplayState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(DisplayHandle::new())))
    }
}

pub struct DisplayHandle {
    monitors: Vec<MonitorWrapper>,
    win32: Win32API,
}

impl DisplayHandle {
    fn new() -> Self {
        let win32 = Win32API::new();

        match win32.get_monitor_device_path() {
            Ok(monitors) => {
                let ws = monitors
                    .into_iter()
                    .map(|x| MonitorWrapper::new(x))
                    .collect::<Vec<MonitorWrapper>>();

                Self {
                    monitors: ws,
                    win32,
                }
            }
            Err(_) => Self {
                monitors: Vec::new(),
                win32,
            },
        }
    }

    pub fn displays(&self) -> Vec<MonitorWrapper> {
        self.monitors.clone()
    }
}

#[command]
pub fn display_info(display: DisplayArg<'_>) -> Result<Vec<MonitorWrapper>, String> {
    let handle = display.0.lock().unwrap();
    // let _ = window.emit("display_info", handle.displays());
    Ok(handle.displays())
}
