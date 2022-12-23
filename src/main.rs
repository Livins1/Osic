#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod ui;


use std;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::mem;
use std::mem::zeroed;
use std::os::windows::prelude::MetadataExt;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use std::ptr;

use imagesize;
use windows::core::PCWSTR;
use windows::core::PWSTR;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::EnumDisplayDevicesA;
use windows::Win32::Graphics::Gdi::EnumDisplayDevicesW;
use windows::Win32::Graphics::Gdi::EnumDisplayMonitors;
use windows::Win32::Graphics::Gdi::EnumDisplaySettingsExA;
use windows::Win32::Graphics::Gdi::GetMonitorInfoW;

use windows::Win32::Foundation::{ERROR_SUCCESS, LUID, NO_ERROR, WIN32_ERROR};
use windows::Win32::Graphics::Gdi::DISPLAY_DEVICEA;
use windows::Win32::Graphics::Gdi::DISPLAY_DEVICEW;
use windows::Win32::Graphics::Gdi::MONITORINFO;
use windows::Win32::Graphics::Gdi::MONITORINFOEXW;
use windows::Win32::Graphics::Gdi::QDC_ONLY_ACTIVE_PATHS;
use windows::Win32::Graphics::Gdi::{HDC, HMONITOR};

use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::DesktopWallpaper;
use windows::Win32::UI::Shell::IDesktopWallpaper;
use windows::Win32::UI::WindowsAndMessaging::EDD_GET_DEVICE_INTERFACE_NAME;

use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, DisplayConfigSetDeviceInfo, GetDisplayConfigBufferSizes,
    QueryDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME,
    DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME, DISPLAYCONFIG_DEVICE_INFO_HEADER,
    DISPLAYCONFIG_DEVICE_INFO_TYPE, DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_MODE_INFO_TYPE,
    DISPLAYCONFIG_MODE_INFO_TYPE_TARGET, DISPLAYCONFIG_PATH_INFO, DISPLAYCONFIG_PATH_TARGET_INFO,
    DISPLAYCONFIG_SOURCE_DEVICE_NAME, DISPLAYCONFIG_SUPPORT_VIRTUAL_RESOLUTION,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, DISPLAYCONFIG_TARGET_MODE,
};

mod utils;

#[repr(C)]
pub struct DISPLAYCONIFG_SOURCE_FRIENDLY_NAME_GET {
    header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
}

fn test_file_find() {
    let path = "C:\\Users\\Tfios\\Pictures\\Background";

    let p = PathBuf::from(path);

    if let Ok(files) = fs::read_dir(p) {
        for file in files {
            let file_entity = file.unwrap();
            println!("file_name: {:?}", file_entity.file_name());
            let file_meta = file_entity.metadata().unwrap();
            println!("file_type : {:?}", file_meta.file_type());
            println!("file_attr : {:?}", file_meta.file_attributes());
            println!("file_path: {:?}", file_entity.path());
            println!(
                "image_reselution: {:?}",
                imagesize::size(file_entity.path())
            );
        }
    }
}

#[derive(Debug)]
pub struct Monitor {
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

fn get_monitor_edid() {
    unsafe {
        let wm =
            CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL).unwrap();

        // 设备路径数量
        let c = wm.GetMonitorDevicePathCount();
        // println!("Count: {}", c.unwrap());
        for i in 0..c.unwrap() {
            let info = wm.GetMonitorDevicePathAt(i);
            // Read PWSTR
            let info = info.unwrap();

            // GetMonitorRECT
            if let Ok(rec) = wm.GetMonitorRECT(PCWSTR(info.as_ptr())) {
                println!("Count: {:?}", &info.to_string().unwrap());
                println!("GetMonitorDevice Rect: {:?}", rec);
            }
        }
    }
}

// good
fn query_display_config() {
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
        // if result != ERROR_SUCCESS as _ {
        //     return Err(std::io::Error::last_os_error())
        //         .context("DisplayConfigGetDeviceInfo DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME");

        // }

        // let mut source_name: DISPLAYCONFIG_TARGET_MODE = unsafe { std::mem::zeroed() };
        // source_name.header.adapterId = path.targetInfo.adapterId;
        // source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        // source_name.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;

        // let result = unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) };

        let mut source_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { std::mem::zeroed() };
        source_name.header.adapterId = path.targetInfo.adapterId;
        source_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
        source_name.header.size = std::mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;

        let result = unsafe { DisplayConfigGetDeviceInfo(&mut source_name.header) };
        // if result != ERROR_SUCCESS as _ {
        //     return Err(std::io::Error::last_os_error())
        //         .context("DisplayConfigGetDeviceInfo DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME");
        // }

        let name = utils::wstr(&target_name.monitorFriendlyDeviceName);
        let device_path = utils::wstr(&target_name.monitorDevicePath);
        let gdi_name = utils::wstr(&source_name.viewGdiDeviceName);
        println!(
            "name: {}, devicePath: {},  gdi_name: {}",
            name, device_path, gdi_name
        );
    }

    for mode in modes {
        if mode.infoType == DISPLAYCONFIG_MODE_INFO_TYPE_TARGET {
            unsafe {
                // DISPLAYCONFIG_PATH_TARGET_INFO
                // Check the resolution

                let signal_info = mode.Anonymous.targetMode.targetVideoSignalInfo;
                // maybe  resolution
                println!(
                    "ActiveSize: {},{}",
                    signal_info.activeSize.cx, signal_info.activeSize.cy
                );
                println!(
                    "TotalSize: {},{}",
                    signal_info.totalSize.cx, signal_info.totalSize.cy
                );

                let desktop_image_info = mode.Anonymous.desktopImageInfo;
                println!("PathSouceSize: {:?}", desktop_image_info.PathSourceSize);
                println!(
                    "Desktop Image Region: {:?}",
                    desktop_image_info.DesktopImageRegion
                );
                println!(
                    "Desktop Image Clip: {:?}",
                    desktop_image_info.DesktopImageClip
                );
                let source_mode = mode.Anonymous.sourceMode;

                println!(
                    "Width: {},Height: {}",
                    &source_mode.width, &source_mode.height
                );
                println!("Souce Mode Position: {:?}", &source_mode.position);
            }
        }
    }
}

fn get_monitor_device_path_count() {
    unsafe { CoInitialize(None).expect("error calling CoInitialize") };
    unsafe {
        let wm =
            CoCreateInstance::<_, IDesktopWallpaper>(&DesktopWallpaper, None, CLSCTX_ALL).unwrap();

        // 设备路径数量
        let c = wm.GetMonitorDevicePathCount();
        // println!("Count: {}", c.unwrap());
        for i in 0..c.unwrap() {
            let info = wm.GetMonitorDevicePathAt(i);
            // Read PWSTR

            let info = info.unwrap();

            // GetMonitorRECT
            if let Ok(rec) = wm.GetMonitorRECT(PCWSTR(info.as_ptr())) {
                println!("Count: {:?}", &info.to_string());
                println!("GetMonitorDevice Rect: {:?}", rec);
            }
        }

        // way2
        let monitors = Box::into_raw(Box::new(Vec::<Monitor>::new()));

        let mut monitors_w = Vec::<MONITORINFOEXW>::new();
        let userdata = &mut monitors_w as *mut _;

        unsafe extern "system" fn enumerate_monitors_callback_w(
            monitor: HMONITOR,
            _: HDC,
            _: *mut RECT,
            userdata: LPARAM,
        ) -> BOOL {
            // Get the userdata where we will store the result
            let monitors: &mut Vec<MONITORINFOEXW> = mem::transmute(userdata);

            // Initialize the MONITORINFOEXW structure and get a pointer to it
            let mut monitor_info: MONITORINFOEXW = mem::zeroed();
            monitor_info.monitorInfo.cbSize = mem::size_of::<MONITORINFO>() as u32;
            let monitor_info_ptr = <*mut _>::cast(&mut monitor_info);

            // Call the GetMonitorInfoW win32 API
            let result = GetMonitorInfoW(monitor, monitor_info_ptr).as_bool();
            if result == true {
                // Push the information we received to userdata
                monitors.push(monitor_info);
            }

            true.into()
        }

        unsafe extern "system" fn enum_monitors_callback(
            handle: HMONITOR,
            _: HDC,
            rect: *mut RECT,
            monitors: LPARAM,
        ) -> BOOL {
            // let monitors = &mut *(monitors as *mut Vec<Monitor>);
            let list = std::mem::transmute::<LPARAM, *mut Vec<Monitor>>(monitors);

            let mut info = MONITORINFOEXW::default();
            info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

            GetMonitorInfoW(handle, ptr::addr_of_mut!(info) as *mut MONITORINFO);

            let rect = *rect;
            // let device_name_raw = OsString::from_wide(&info.szDevice);
            // let device_name_length = device
            //     .DeviceKey
            //     .iter()
            //     .position(|&x| x == 0)
            //     .expect("Unterminated text");

            // let name: String = OsString::from_wide(&device.DeviceKey[..device_name_length])
            //     .to_string_lossy()
            //     .to_string();
            let name = match &info.szDevice[..].iter().position(|c| *c == 0) {
                Some(len) => OsString::from_wide(&info.szDevice[0..*len]),
                None => OsString::from_wide(&info.szDevice[0..info.szDevice.len()]),
            };

            println!("Rect: {:?}", rect);
            (*list).push(Monitor {
                name: name.to_str().unwrap().to_string(),
                x: rect.left,
                y: rect.top,
                width: (rect.right - rect.left) as u32,
                height: (rect.bottom - rect.top) as u32,
            });

            true.into()
        }

        //
        if EnumDisplayMonitors(
            HDC(0),
            None,
            Some(enum_monitors_callback),
            LPARAM(monitors as isize),
        )
        .as_bool()
        {
            for monitor in *Box::from_raw(monitors) {
                println!("MonitorName: {}", monitor.name);
                println!(
                    "MonitorSize: width {}, height {}",
                    monitor.width, monitor.height
                );
            }
        }

        if EnumDisplayMonitors(
            HDC(0),
            None,
            Some(enumerate_monitors_callback_w),
            LPARAM(userdata as isize),
        )
        .as_bool()
        {
            for monitor in monitors_w {
                let name = match &monitor.szDevice[..].iter().position(|c| *c == 0) {
                    Some(len) => OsString::from_wide(&monitor.szDevice[0..*len]),
                    None => OsString::from_wide(&monitor.szDevice[0..monitor.szDevice.len()]),
                };

                // Print some information to the console
                println!("Display name = {}", name.to_str().unwrap());
                println!("    Left: {}", monitor.monitorInfo.rcMonitor.left);
                println!("   Right: {}", monitor.monitorInfo.rcMonitor.right);
                println!("     Top: {}", monitor.monitorInfo.rcMonitor.top);
                println!("  Bottom: {}", monitor.monitorInfo.rcMonitor.bottom);
            }
        }

        // way 3
        // enum DisplayDevice name
        let mut device = DISPLAY_DEVICEW::default();
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;

        // this Enum the displayDevice like "AMD Radeon RX 6600"
        if EnumDisplayDevicesW(None, 0, &mut device, 0).as_bool() {
            let device_name = device.DeviceName;

            let name = match &device.DeviceString[..].iter().position(|c| *c == 0) {
                Some(len) => OsString::from_wide(&device.DeviceString[0..*len]),
                None => OsString::from_wide(&device.DeviceString[0..device.DeviceString.len()]),
            };
            println!("DisplayDevice: {:?}", &name);
            if EnumDisplayDevicesW(PCWSTR(device_name[..].as_ptr()), 0, &mut device, 0).as_bool() {
                // let device_name_length = device
                //     .DeviceKey
                //     .iter()
                //     .position(|&x| x == 0)
                //     .expect("Unterminated text");

                // let name: String = OsString::from_wide(&device.DeviceKey[..device_name_length])
                //     .to_string_lossy()
                //     .to_string();

                let name = match &device.DeviceString[..].iter().position(|c| *c == 0) {
                    Some(len) => OsString::from_wide(&device.DeviceString[0..*len]),
                    None => OsString::from_wide(&device.DeviceString[0..device.DeviceString.len()]),
                };
                println!("MnitorDevice: {:?}", &name);
            }
        } else {
            println!("EnumDisplayDevice Error");
        }
    }
}

fn test_monitor_function() {
    get_monitor_device_path_count();
    query_display_config()
}

fn main() {
    // test_monitor_function()
    ui::ui_init()
}
