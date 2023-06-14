use async_std::task;
use async_std::task::block_on;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use tauri::{command, State, Window};

// use image::{GenericImageView, ImageBuffer, Pixel, Rgba};
use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;

use crate::cache::AppCache;
use crate::utils;

pub struct GalleryState(Arc<Mutex<Gallery>>);
pub type GalleryArg<'a> = State<'a, GalleryState>;

impl GalleryState {
    // pub fn new() -> Self {
    //     Self(Arc::new(Mutex::new(Gallery::new())))
    // }
    pub fn new(c: AppCache) -> Self {
        Self(Arc::new(Mutex::new(Gallery::new(c).cached())))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreviewPicture {
    picture: Picture,
    thumbnail: String,
}

// #[derive(Clone, Copy)]
pub struct Gallery {
    folders: Vec<Folder>,
    cache: AppCache,
}

impl Gallery {
    fn new(c: AppCache) -> Self {
        Gallery {
            folders: Vec::new(),
            cache: c,
        }
    }

    fn cached(mut self) -> Self {
        println!("Start Load cache..");

        let start = Instant::now();
        if let Ok(Some(s)) = self.cache.read("folders") {
            self.folders = serde_json::from_str(s.as_str()).unwrap();
            self.folders.iter_mut().for_each(|x| x.async_scan());
        };

        println!("Cache Load Finished: {:?}", start.elapsed(),);
        self
    }

    fn save_cache(&self) {
        if let Ok(s) = serde_json::to_string(&self.folders) {
            let _ = self.cache.write("folders", s);
        }
    }

    fn remove_folder(&mut self, index: usize) {
        self.folders.remove(index);

        for (i, item) in self.folders.iter_mut().enumerate() {
            item.index = i
        }
    }

    fn new_folder(&mut self, path: String) {
        self.folders.push(Folder::new(path, self.folders.len()));

        if let Some(folder) = self.folders.last_mut() {
            folder.loading = true;
            // folder.scan();
            folder.async_scan();
        }
        self.save_cache();
    }

    fn rescan_folder(&mut self, index: usize) {
        self.folders[index].async_scan();
        self.save_cache();
    }

    fn preview(
        &self,
        page: usize,
        size: usize,
        path_index: i32,
    ) -> Result<Vec<PreviewPicture>, String> {
        if let Some(folder) = self.folders.get(path_index as usize) {
            let pictures: Vec<PreviewPicture> = folder
                .select_pictures(page, size)
                .into_par_iter()
                .map(|x| x.to_preview(&self.cache))
                .collect();

            return Ok(pictures);
        }
        Ok(Vec::new())
    }

    // fn show_folder_lsit(&self) {
    //     self.folders.iter().for_each(|x| println!("{:?}", x.path));
    // }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Folder {
    path: String,
    index: usize,
    loading: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pictures: Vec<Picture>,
    quanitity: usize,
}

impl Folder {
    fn new(path: String, index: usize) -> Folder {
        Folder {
            path: path,
            index,
            loading: true,
            quanitity: 0,
            pictures: Vec::new(),
        }
    }
    fn async_scan(&mut self) {
        let folder = fs::read_dir(&self.path).unwrap();

        let start = Instant::now();
        let mut tasks = Vec::new();
        for entry in folder {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "jpg" || extension == "jpeg" || extension == "png" {
                        let task = task::spawn(
                            async move {
                                let hash = utils::hash_file(path.clone().into()).await.unwrap();

                                Picture::new(path, hash)
                            }, // utils::hash_file(path.clone().into())
                        );
                        tasks.push(task);
                        // println!("hash: {:?}", hash);

                        // self.pictures.push(Picture::new(path))
                    }
                }
            }
        }
        // let results = futures::future::join_all(tasks).await;
        let result = block_on(futures::future::join_all(tasks));
        self.pictures = result;
        self.quanitity = self.pictures.len();

        let duration = start.elapsed();
        println!(
            "Scan duration: {:?}, Pictures: {:?}",
            duration,
            self.pictures.len()
        )
    }

    // scan pictures of this Folder
    fn scan(&mut self) {
        let folder = fs::read_dir(&self.path).unwrap();

        let start = Instant::now();
        for entry in folder {
            if let Ok(entry) = entry {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "jpg" || extension == "jpeg" || extension == "png" {
                        let hash = utils::generate_hash(path.to_str().unwrap());
                        // println!("hash: {:?}", hash);
                        println!(
                            "Hash generation duration: {:?}, hash value: {:?}",
                            start.elapsed(),
                            hash
                        );

                        self.pictures.push(Picture::new(path, String::new()))
                    }
                }
            }
        }
        self.quanitity = self.pictures.len();

        let duration = start.elapsed();
        println!(
            "Scan duration: {:?}, Pictures: {:?}",
            duration,
            self.pictures.len()
        )
    }

    // write pictures cache
    fn write_cache(&self) {}

    fn select_pictures(&self, page: usize, size: usize) -> Vec<Picture> {
        let start_index = page * size;
        if self.pictures.len() == 0 || start_index > self.pictures.len() - 1 {
            return Vec::new();
        }

        let end_index = std::cmp::min(start_index + size, self.pictures.len());

        let slice = self.pictures[start_index..end_index].to_vec();
        return slice;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Picture {
    path: PathBuf,
    hash: String,
    ratio: Option<f32>,
    reselution: Option<(i32, i32)>,
}

impl Picture {
    fn new(path: PathBuf, hash: String) -> Picture {
        Picture {
            path: path,
            hash: hash,
            ratio: Some(3.12),
            reselution: Some((12, 12)),
        }
    }

    fn to_preview(&self, c: &AppCache) -> PreviewPicture {
        let src_prefix = match self.path.extension().unwrap().to_str() {
            Some("jpg") => "data:image/jpg;base64,",
            Some("png") => "data:image/png;base64,",
            Some("jpeg") => "data:image/jpg;base64,",
            _ => "",
        };
        // let thumbnail_cache = c.read_thumbnail(self.hash)
        if let Ok(Some(t)) = c.read_thumbnail(self.hash.as_str()) {
            return PreviewPicture {
                picture: self.clone(),
                thumbnail: t,
            };
        }

        PreviewPicture {
            picture: self.clone(),
            // thumbnail: String::new(),
            thumbnail: match self.thumbnail(256, src_prefix) {
                Some(s) => {
                    _ = c.save_thumbnail(&self.hash, s.clone());
                    s
                }
                None => String::new(),
            },
        }
    }

    fn thumbnail(&self, max_px: u32, prefix: &str) -> Option<String> {
        // let extension = self.path.extension()?.to_str()?;
        let start = Instant::now();
        let image = match image::open(self.path.as_path()) {
            Ok(image) => image,
            Err(_) => return None,
        };
        println!("OpenImage: {:?}", start.elapsed(),);

        // let resize_ratio = std::cmp::max(image.width(), image.height()) / max_px as u32;
        // let (width, height) = (image.width() / resize_ratio, image.height() / resize_ratio);
        let thumbnail = image.thumbnail(max_px, max_px);

        let duration = start.elapsed();

        println!(
            "thumbnail generation duration : {:?}, width: {:?}, height: {:?}",
            duration,
            image.width(),
            image.height()
        );
        use std::io::Cursor;
        let mut buffer = Cursor::new(Vec::new());

        thumbnail
            .write_to(&mut buffer, image::ImageOutputFormat::Png)
            .unwrap();

        return Some(
            prefix.to_string() + &general_purpose::STANDARD_NO_PAD.encode(buffer.get_ref()),
        );
    }
}

#[command]
pub fn add_folder(path: &str, gallery: GalleryArg<'_>, window: Window) -> String {
    let mut gallery = gallery.0.lock().unwrap();
    gallery.new_folder(String::from(path));

    // update Frontend
    window
        .emit("update_folders", gallery.folders.clone())
        .unwrap();
    String::from("Success")
}

#[command]
pub fn get_folders(gallery: GalleryArg<'_>) -> Result<Vec<Folder>, String> {
    let gallery = gallery.0.lock().unwrap();
    Ok(gallery.folders.clone())
}

#[command]
pub fn remove_folder(gallery: GalleryArg<'_>, index: usize, window: Window) -> Result<(), String> {
    let mut gallery = gallery.0.lock().unwrap();
    gallery.remove_folder(index);

    window
        .emit("update_folders", gallery.folders.clone())
        .unwrap();
    Ok(())
}

#[command]
pub fn rescan_folder(gallery: GalleryArg<'_>, index: usize, window: Window) -> Result<(), String> {
    let mut gallery = gallery.0.lock().unwrap();
    gallery.rescan_folder(index);
    window
        .emit("update_folders", gallery.folders.clone())
        .unwrap();
    Ok(())
}

#[command]
pub fn preview(
    gallery: GalleryArg<'_>,
    page: usize,
    size: usize,
    folder_index: i32,
) -> Result<Vec<PreviewPicture>, String> {
    let gallery = gallery.0.lock().unwrap();
    println!(
        "Gallery Prewview Page: {:?}, FolderIndex: {:?}",
        page, folder_index
    );
    gallery.preview(page, size, folder_index)

    // Ok(())
}

#[command]
pub fn explorer_file(path: &str) -> Result<(), String> {
    let _ = utils::file_locate_exploer(path);
    Ok(())
}
