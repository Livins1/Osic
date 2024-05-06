use image::{imageops::FilterType, *};
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::PathBuf;

// pub fn imgbuff_to_egui_imgdata<P, Container>(i: ImageBuffer<P, Container>) -> ImageData
// where
//     P: Pixel<Subpixel = u8>,
//     Container: Deref<Target = [P::Subpixel]> + std::convert::AsRef<[<P as image::Pixel>::Subpixel]>,
// {
//     let size = [i.width() as _, i.height() as _];
//     let pixels: image::FlatSamples<&[u8]> = i.as_flat_samples();
//     ImageData::from(ColorImage::from_rgba_premultiplied(size, pixels.as_slice()))
// }

pub fn gen_gallery(img_path: &PathBuf, size_width: i32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let img = image::open(img_path.as_path()).unwrap();
    let (t_w, t_h) = (img.width() as i32, img.height() as i32);

    let mut resize_ratio = t_w.min(t_h) / size_width;
    if resize_ratio <= 2 {
        resize_ratio = 1
    }
    println!("Gen Gallery resize_ratio: {}", resize_ratio);

    let (r_w, r_h) = (t_w / resize_ratio, t_h / resize_ratio);

    let mut i = match resize_ratio > 2 {
        true => img.resize_exact(r_w as u32, r_h as u32, FilterType::Triangle),
        false => img,
    };
    // let mut i = img.resize_exact(r_w as u32, r_h as u32, FilterType::Triangle);
    if r_w - r_h >= 0 {
        let x = (r_w - r_h) / 2;
        return sub_img(&mut i, x, 0, r_h, r_h).to_image();
    } else {
        let y = (r_h - r_w) / 2;
        return sub_img(&mut i, 0, y, r_w, r_w).to_image();
    };
}

pub fn sub_img(
    img: &mut DynamicImage,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> SubImage<&mut DynamicImage> {
    let sub_img = imageops::crop(img, x as u32, y as u32, w as u32, h as u32);
    return sub_img;
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn file_hash(path: &PathBuf) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = [0; 1024];
    file.read_exact(&mut contents)?;
    let mut hasher = DefaultHasher::new();
    contents.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

pub fn string_hash(s: String) -> io::Result<String> {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    Ok(format!("{:x}", hasher.finish()))
}

use std::time::SystemTime;

pub fn get_sys_time_in_secs() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
