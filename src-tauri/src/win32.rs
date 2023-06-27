use std;

use std::mem::zeroed;

use crate::utils;
use std::time::Instant;

use windows::core::HSTRING;
use windows::core::PCWSTR;

use windows::Win32::Foundation::ERROR_SUCCESS;

use windows::Win32::Devices::Display::QDC_ONLY_ACTIVE_PATHS;

use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::DesktopWallpaper;
use windows::Win32::UI::Shell::IDesktopWallpaper;

use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_TARGET_DEVICE_NAME,
};

#[derive(Debug)]
pub struct Monitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// good
fn query_display_config() -> Result<Vec<(String, String, String)>, String> {
    let mut num_paths: u32 = 0;
    let mut paths: [DISPLAYCONFIG_PATH_INFO; 32] = unsafe { zeroed() };
    let mut num_modes: u32 = 0;
    let mut modes: [DISPLAYCONFIG_MODE_INFO; 32] = unsafe { zeroed() };

    let ret = unsafe {
        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes)
    };
    if ret != ERROR_SUCCESS {
        return Err(format!(
            "GetDisplayConfigBufferSizes Failed: Win32_ERROR({})",
            ret.0
        ));
    }

    let ret = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            None,
            // std::ptr::null_mut(),
        )
    };
    if ret != ERROR_SUCCESS {
        return Err(format!("QueryDisplayConfig Failed: Win32_ERROR({})", ret.0));
    }
    let mut r: Vec<(String, String, String)> = Vec::new();

    for path in &paths {
        let mut target_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { std::mem::zeroed() };

        target_name.header.adapterId = path.targetInfo.adapterId;
        target_name.header.id = path.targetInfo.id;
        target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        target_name.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;

        let _ = unsafe { DisplayConfigGetDeviceInfo(&mut target_name.header) };
        let mut source_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { std::mem::zeroed() };
        source_name.header.adapterId = path.targetInfo.adapterId;
        source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_name.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;

        let _ = unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) };

        let name = utils::wstr(&target_name.monitorFriendlyDeviceName);
        let device_path = utils::wstr(&target_name.monitorDevicePath);
        let gdi_name = utils::wstr(&source_name.viewGdiDeviceName);
        if name.len() > 0 {
            r.push((name, device_path, gdi_name))
        }
    }
    Ok(r)
}

// pub fn get_monitors() -> Result<Vec<crate::monitor::Monitor>, String> {
//     let mut monitors: Vec<crate::monitor::Monitor> = Vec::new();
//     unsafe {
//         if CoInitialize(None).is_err() {
//             return Err("CoInitialize Error".to_string());
//         }
//     };
//     unsafe {
//         let wm =
//             CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL).unwrap();

//         // device count
//         let c = wm.GetMonitorDevicePathCount();
//         for i in 0..c.unwrap() {
//             let info = wm.GetMonitorDevicePathAt(i);
//             let info = info.unwrap();

//             // GetMonitorRECT
//             if let Ok(rec) = wm.GetMonitorRECT(PCWSTR(info.as_ptr())) {
//                 println!("Count: {:?}", &info.to_string());
//                 println!("GetMonitorDevice Rect: {:?}", rec);
//                 let id = info
//                     .to_string()
//                     .expect("error calling monitor device info to_string");

//                 monitors.push(crate::monitor::Monitor::from_win32(
//                     "".to_string(),
//                     id,
//                     "".to_string(),
//                     rec,
//                 ))
//             }
//         }

//         if monitors.len() > 0 {
//             match query_display_config() {
//                 Ok(configs) => {
//                     for (name, device_path, gdi_name) in configs {
//                         if let Some(monitor) = monitors.iter_mut().find(|m| m.id == device_path) {
//                             monitor.title = name;
//                             monitor.gdi_name = gdi_name;
//                         }
//                     }
//                 }
//                 Err(e) => return Err(e),
//             }
//         }
//         Ok(monitors)
//     }
// }

pub struct Win32API {
    wm: IDesktopWallpaper,
}
// here: https://stackoverflow.com/questions/60292897/why-cant-i-send-mutexmut-c-void-between-threads 
unsafe impl Send for Win32API {}


impl Win32API {
    pub fn new() -> Self {
        unsafe {
            CoInitialize(None).expect("CoInitialize Error");
            let wm = CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL)
                .unwrap();
            Self { wm }
        }
    }

    pub fn get_monitors(&self) -> Result<Vec<crate::monitor::Monitor>, String> {
        let mut monitors: Vec<crate::monitor::Monitor> = Vec::new();
        unsafe {
            // device count
            let c = self.wm.GetMonitorDevicePathCount();
            for i in 0..c.unwrap() {
                let info = self.wm.GetMonitorDevicePathAt(i);
                let info = info.unwrap();

                // GetMonitorRECT
                if let Ok(rec) = self.wm.GetMonitorRECT(PCWSTR(info.as_ptr())) {
                    let id = info
                        .to_string()
                        .expect("error calling monitor device info to_string");

                    monitors.push(crate::monitor::Monitor::from_win32(
                        "".to_string(),
                        id,
                        "".to_string(),
                        rec,
                    ))
                }
            }

            if monitors.len() > 0 {
                match query_display_config() {
                    Ok(configs) => {
                        for (name, device_path, gdi_name) in configs {
                            if let Some(monitor) = monitors.iter_mut().find(|m| m.id == device_path)
                            {
                                monitor.title = name;
                                monitor.gdi_name = gdi_name;
                            }
                        }
                    }
                    Err(e) => return Err(e),
                }
            }
            Ok(monitors)
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
}

pub fn test_monitor_function() {
    let start = Instant::now();

    let w32api = Win32API::new();

    if let Ok(b) = w32api.get_monitors() {
        println!("Monitors : {:?}", b)
    }

    println!("test_monitor_function end: {:?}", start.elapsed(),);
}
