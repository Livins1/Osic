
use windows::core::{HSTRING, PCWSTR};

use serde::{Deserialize, Serialize};
// use std::{clone, mem};
use std::mem::zeroed;
// use std::os::raw::c_void;

use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, QDC_ONLY_ACTIVE_PATHS,
};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::DesktopWallpaper;
use windows::Win32::UI::Shell::IDesktopWallpaper;

use crate::cache::{self, OsicMonitorSettings};
use crate::utils;

#[derive(Clone)]
pub struct Win32API {
    wm: IDesktopWallpaper,
    // workw: Option<Box<HWND>>,
    com_err: bool,
}
// here: https://stackoverflow.com/questions/60292897/why-cant-i-send-mutexmut-c-void-between-threads
unsafe impl Send for Win32API {}

// unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
//     // let closure: &mut &mut dyn FnMut(HWND) -> bool = mem::transmute(lparam ) as *mut c_void as *mut _;
//     let closure: &mut &mut dyn FnMut(HWND) -> bool = {
//         // let c = lparam as *mut c_void;
//         let c: *mut c_void = mem::transmute(lparam);
//         &mut *(c as *mut _)
//     };
//     if closure(hwnd) {
//         TRUE
//     } else {
//         FALSE
//     }
// }

impl Win32API {
    pub fn new() -> Self {
        unsafe {
            let com_err = CoInitialize(None).is_err();
            let wm = CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL)
                .unwrap();
            Self {
                wm,
                // workw: None,
                com_err,
            }
        }
    }

    pub fn set_wallpaper(
        &self,
        monitor_id: &str,
        wallpaper: &str,
    ) -> Result<(), windows::core::Error> {
        unsafe {
            self.wm.SetWallpaper(
                PCWSTR::from_raw(HSTRING::from(monitor_id).as_ptr()),
                PCWSTR::from_raw(HSTRING::from(wallpaper).as_ptr()),
            )
        }
    }

    pub fn set_fit(&self, fits: i32) -> Result<(), windows::core::Error> {
        let i = windows::Win32::UI::Shell::DESKTOP_WALLPAPER_POSITION(fits);
        unsafe { self.wm.SetPosition(i) }
    }

    pub fn get_monitor_device_path(&self) -> Result<Vec<Monitor>, String> {
        let mut monitors: Vec<Monitor> = Vec::<Monitor>::new();

        unsafe {
            let wm = &self.wm;

            let c = wm.GetMonitorDevicePathCount();
            for i in 0..c.unwrap() {
                let mut monitor = Monitor::default();
                let info = wm.GetMonitorDevicePathAt(i);
                // Read PWSTR

                let info = info.unwrap();

                // GetMonitorRECT
                if let Ok(rec) = wm.GetMonitorRECT(PCWSTR(info.as_ptr())) {
                    println!("Count: {:?}", &info.to_string());
                    println!("GetMonitorDevice Rect: {:?}", rec);

                    if let Ok(i) = info.to_string() {
                        monitor.device_id = i;
                        monitor.bottom = rec.bottom;
                        monitor.letf = rec.left;
                        monitor.right = rec.right;
                        monitor.top = rec.top;
                        monitor.width = (rec.right - rec.left).abs();
                        monitor.height = (rec.top - rec.bottom).abs();
                        monitors.push(monitor);
                    }
                }
            }
        }

        // 读取显示路径
        let mut num_paths: u32 = 0;
        let mut paths: [DISPLAYCONFIG_PATH_INFO; 32] = unsafe { zeroed() };

        let mut num_modes: u32 = 0;
        let mut modes: [DISPLAYCONFIG_MODE_INFO; 32] = unsafe { zeroed() };
        // DISPLAYCONFIG_MODE_INFO

        let ret = unsafe {
            GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
        };
        if WIN32_ERROR(ret.0) != ERROR_SUCCESS {
            return Err(format!("GetDisplayConfigBufferSizes WIN32 Error:{}", ret.0));
        }

        let ret = unsafe {
            QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut num_paths,
                paths.as_mut_ptr(),
                &mut num_modes,
                modes.as_mut_ptr(),
                None, // std::ptr::null_mut(),
            )
        };
        if WIN32_ERROR(ret.0) != ERROR_SUCCESS {
            return Err(format!("QueryDisplayConfig WIN32 Error:{}", ret.0));
        }

        for path in &paths {
            let mut target_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { std::mem::zeroed() };

            target_name.header.adapterId = path.targetInfo.adapterId;
            target_name.header.id = path.targetInfo.id;
            target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
            target_name.header.size =
                std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;

            let _ = unsafe { DisplayConfigGetDeviceInfo(&mut target_name.header) };

            let name = utils::wstr(&target_name.monitorFriendlyDeviceName);
            let device_path = utils::wstr(&target_name.monitorDevicePath);
            for monitor in &mut monitors {
                if monitor.device_id == device_path {
                    monitor.name = name.clone();
                }
            }
        }

        return Ok(monitors);
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
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

impl Monitor {
    pub fn load_from_cache(&self) -> Result<OsicMonitorSettings, ()> {
        cache::load_monitor_settings(self.device_id.clone())
    }
}
