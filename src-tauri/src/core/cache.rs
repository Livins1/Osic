use crate::core::display::{Fits, Interval, Modes, MonitorWrapper};
use crate::core::selector::OsicSlideSelector;
use crate::core::utils;
use image::{DynamicImage, ImageBuffer, ImageError};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self, File},
    io::{self, BufReader},
    path::PathBuf,
};

fn os_temp_folder() -> PathBuf {
    let dir = env::temp_dir();
    return dir.join("Osic");
}

fn app_tmp_image_path() -> PathBuf {
    let mut p = os_temp_folder();
    p.push("img");
    let _ = fs::create_dir_all(&p);
    return p;
}

pub fn write_image_cache(
    image_path: &PathBuf,
    content: &ImageBuffer<image::Rgba<u8>, Vec<u8>>,
) -> Result<(), ImageError> {
    let mut img_path = app_tmp_image_path();
    let file_name = utils::file_hash(&image_path).unwrap();
    img_path.push(file_name);
    println!("Temporary image directory: {}", &img_path.display());
    if img_path.exists() {
        return Ok(());
    };

    match image::save_buffer_with_format(
        img_path,
        content,
        content.width(),
        content.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Jpeg,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn load_image_cache(image_path: &PathBuf) -> Result<DynamicImage, ()> {
    let mut img_path = app_tmp_image_path();
    let file_name = utils::file_hash(image_path).unwrap();
    img_path.push(file_name);
    if let Ok(f) = File::open(img_path) {
        let reader = BufReader::new(f);
        match image::load(reader, image::ImageFormat::Jpeg) {
            Ok(s) => Ok(s),
            Err(_) => Err(()),
        }
    } else {
        Err(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OsicRecentImage {
    pub path: PathBuf,
    // pub thumbnail: ImageData,
    // pub thumbnail_texture: TextureHandle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OsicMonitorSettings {
    pub device_id: String,
    pub mode: Modes,
    pub fit: Fits,
    pub image: Option<PathBuf>,
    pub album_path: Option<PathBuf>,
    pub recent_images: Vec<PathBuf>,
    pub slide_interval: Interval,
    pub slide_time: u64,
    pub selector: OsicSlideSelector,
}

impl From<MonitorWrapper> for OsicMonitorSettings {
    fn from(item: MonitorWrapper) -> Self {
        let recent_images = item.image_history.into_iter().map(|i| i.path).collect();

        OsicMonitorSettings {
            device_id: item.device_id,
            mode: item.mode,
            album_path: item.album_path,
            fit: item.fit,
            image: item.image,
            recent_images: recent_images,
            slide_interval: item.slide_interval,
            slide_time: item.slide_time,
            selector: item.selector,
        }
    }
}

pub fn write_monitor_settings(s: OsicMonitorSettings) -> Result<(), io::Error> {
    let encode_struct = bincode::serialize(&s).unwrap();

    let file_name = utils::string_hash(s.device_id).unwrap();
    let mut file_path = os_temp_folder();
    file_path.push(file_name);

    if file_path.exists() {
        fs::remove_file(&file_path)?;
    };

    fs::write(file_path, encode_struct)
}

pub fn load_monitor_settings(device_id: String) -> Result<OsicMonitorSettings, ()> {
    let file_name = utils::string_hash(device_id).unwrap();
    let mut file_path = os_temp_folder();
    file_path.push(file_name);

    if !file_path.exists() {
        return Err(());
    };

    match fs::read(file_path) {
        Ok(s) => {
            let decoded_struct = bincode::deserialize(&s).unwrap();
            return Ok(decoded_struct);
        }
        Err(_) => Err(()),
    }
}
