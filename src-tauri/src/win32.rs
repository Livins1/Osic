use std;

use std::mem;
use std::mem::zeroed;
use std::os::raw::c_void;
use std::os::windows::io::NullHandleError;

use crate::utils;
use std::time::Instant;

use windows::core::{HSTRING, PCSTR, PCWSTR};

use windows::Win32::Foundation::{BOOL, ERROR_SUCCESS, FALSE, HWND, LPARAM, TRUE, WPARAM};

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

use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowA, FindWindowExA, FindWindowExW, SendMessageTimeoutA, SetParent,
    SMTO_NORMAL,
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

unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // let closure: &mut &mut dyn FnMut(HWND) -> bool = mem::transmute(lparam ) as *mut c_void as *mut _;
    let closure: &mut &mut dyn FnMut(HWND) -> bool = {
        // let c = lparam as *mut c_void;
        let c: *mut c_void = mem::transmute(lparam);
        &mut *(c as *mut _)
    };
    if closure(hwnd) {
        TRUE
    } else {
        FALSE
    }
}
pub struct Win32API {
    wm: IDesktopWallpaper,
    workw: Option<Box<HWND>>,
}
// here: https://stackoverflow.com/questions/60292897/why-cant-i-send-mutexmut-c-void-between-threads
unsafe impl Send for Win32API {}

impl Win32API {
    pub fn new() -> Self {
        unsafe {
            CoInitialize(None).expect("CoInitialize Error");
            let wm = CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL)
                .unwrap();
            Self { wm, workw: None }
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

    pub fn init_workw(&mut self) -> Result<(), String> {
        // let mut workw = Box::into_raw(Box::new(HWND::default()));

        unsafe {
            let progman = FindWindowA(PCSTR::from_raw("Progman".as_ptr()), None);

            SendMessageTimeoutA(
                progman,
                0x052C,
                WPARAM::default(),
                LPARAM::default(),
                SMTO_NORMAL,
                1000,
                None,
            );
            let mut workw = HWND::default();

            let mut enum_windows_proc = |window: HWND| -> bool {
                let p = FindWindowExA(
                    window,
                    HWND::default(),
                    PCSTR::from_raw("SHELLDLL_DefView".as_ptr()),
                    None,
                );

                // We find that window
                if p != HWND(0) {
                    println!("SHELLDLL_DefView handle found.");

                    // Use FindWindowExW can find WorkerW, but not FindWindowExA
                    workw = FindWindowExW(
                        None,
                        window,
                        PCWSTR::from_raw(HSTRING::from("WorkerW").as_ptr()),
                        None,
                    );
                };

                return true;
            };

            let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut enum_windows_proc;
            let trait_obj_ref = &mut trait_obj;
            let closure_pinter_pinter = trait_obj_ref as *mut _ as *mut c_void;

            let lparam = closure_pinter_pinter as isize;
            EnumWindows(Some(enumerate_callback), LPARAM(lparam));

            if workw == std::mem::zeroed() {
                println!("Failed to setup WorkerW, handle not found ....");
                return Err("WorkerW handle not found ...".to_string());
            } else {
                println!("WorkerW initialized ..");
                self.workw = Some(Box::new(workw));
                return Ok(());
            }
        }
        // Ok(())
    }
}

pub fn test_monitor_function() {
    let start = Instant::now();

    let mut w32api = Win32API::new();

    // if let Ok(b) = w32api.get_monitors() {
    //     println!("Monitors : {:?}", b)
    // }
    if let Err(s) = w32api.init_workw() {
        println!("init_Workw : {:?}", s)
    }

    println!("test_monitor_function end: {:?}", start.elapsed(),);
}
