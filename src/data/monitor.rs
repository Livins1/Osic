use std::ffi::{OsStr, OsString};
use std::mem::zeroed;

use crate::utils;
use windows::core::PCWSTR;
use windows::core::PWSTR;
use windows::Win32::Foundation::{ERROR_SUCCESS, LUID, NO_ERROR, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::QDC_ONLY_ACTIVE_PATHS;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::DesktopWallpaper;
use windows::Win32::UI::Shell::IDesktopWallpaper;

use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, GetDisplayConfigBufferSizes, QueryDisplayConfig,
    DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_SOURCE_DEVICE_NAME,
    DISPLAYCONFIG_TARGET_DEVICE_NAME,
};

#[derive(Debug, Default)]
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

pub fn get_monitor_device_path() {
    let mut monitors: Vec<Monitor> = Vec::<Monitor>::new();

    unsafe { CoInitialize(None).expect("error calling CoInitialize") };
    unsafe {
        let wm =
            CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL).unwrap();

        // 设备路径数量
        let c = wm.GetMonitorDevicePathCount();
        // println!("Count: {}", c.unwrap());
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
    if WIN32_ERROR(ret as u32) != ERROR_SUCCESS {
        println!("GetDisplayConfigBufferSizes Failed: WIN32_ERROR({})", ret);
    }

    let ret = unsafe {
        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            std::ptr::null_mut(),
        )
    };
    if WIN32_ERROR(ret as u32) != ERROR_SUCCESS {
        println!("QueryDisplayConfig Failed: WIN32_ERROR({})", ret)
    }

    for path in &paths {
        let mut target_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { std::mem::zeroed() };

        target_name.header.adapterId = path.targetInfo.adapterId;
        target_name.header.id = path.targetInfo.id;
        target_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        target_name.header.size = std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;

        let result = unsafe { DisplayConfigGetDeviceInfo(&mut target_name.header) };

        let mut source_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { std::mem::zeroed() };
        source_name.header.adapterId = path.targetInfo.adapterId;
        source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_name.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;

        // let result = unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) };
        // if result != ERROR_SUCCESS as _ {
        //     return Err(std::io::Error::last_os_error())
        //         .context("DisplayConfigGetDeviceInfo DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME");
        // }
        let name = utils::wstr(&target_name.monitorFriendlyDeviceName);
        let device_path = utils::wstr(&target_name.monitorDevicePath);
        // let gdi_name = utils::wstr(&source_name.viewGdiDeviceName);
        for monitor in &mut monitors {
            if monitor.device_id == device_path {
                monitor.name = name.clone();
                print!("{:?}", monitor);
            }
        }

    }
}
