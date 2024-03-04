use std::{
    env,
    fs::{self, File, OpenOptions},
    io::{self, BufReader, Write},
    path::PathBuf,
};

use egui::{ImageData, TextureHandle};
use image::{DynamicImage, ImageBuffer, ImageError};

use crate::utils;

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

#[derive(Clone)]
pub struct OsicRecentImage {
    pub path: PathBuf,
    pub thumbnail: ImageData,
    pub thumbnail_texture: TextureHandle,
}
