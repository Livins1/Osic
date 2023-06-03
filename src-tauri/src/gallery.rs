use async_std::task;
use async_std::task::block_on;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use tauri::{command, Event, State, Window};

// use image::{GenericImageView, ImageBuffer, Pixel, Rgba};
use base64::{engine::general_purpose, Engine as _};
use rayon::prelude::*;

use crate::cache::AppCache;
use crate::utils;

pub struct GalleryState(Arc<Mutex<Gallery>>);
pub type GalleryArg<'a> = State<'a, GalleryState>;

impl GalleryState {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Gallery::new())))
    }
    // pub fn new(c: AppCache) -> Self {
    //     Self(Arc::new(Mutex::new(Gallery::new(c))))
    // }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PreviewPicture {
    picture: Picture,
    thumbnail: String,
}

pub struct Gallery {
    folders: Vec<Folder>,
    // Cache: AppCache,
}

impl Gallery {
    // fn new(c: AppCache) -> Gallery {
    //     Gallery {
    //         folders: Vec::new(),
    //         Cache: c,
    //     }
    // }
    fn new() -> Gallery {
        Gallery {
            folders: Vec::new(),
            // Cache: c,
        }
    }

    fn new_folder(&mut self, path: String) {
        self.folders
            .push(Folder::new(path, self.folders.len() as i32));

        if let Some(folder) = self.folders.last_mut() {
            folder.loading = true;
            // folder.scan();
            folder.async_scan();
        }
    }

    fn preview(
        &self,
        page: usize,
        size: usize,
        path_index: i32,
    ) -> Result<Vec<PreviewPicture>, String> {
        // let folder = self.folders.get(path_index);
        if let Some(folder) = self.folders.get(path_index as usize) {
            // let pictures: Vec<PreviewPicture> = folder
            //     .select_pictures(page, size)
            //     .iter()
            //     .map(|x| x.to_preview())
            //     .collect();

            let pictures: Vec<PreviewPicture> = folder
                .select_pictures(page, size)
                .into_par_iter()
                .map(|x| x.to_preview())
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
    index: i32,
    loading: bool,

    #[serde(skip_serializing)]
    pictures: Vec<Picture>,
}

impl Folder {
    fn new(path: String, index: i32) -> Folder {
        Folder {
            path: path,
            index,
            loading: true,
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

        let duration = start.elapsed();
        println!(
            "Scan duration: {:?}, Pictures: {:?}",
            duration,
            result.len()
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

        let end_index = std::cmp::min(start_index + size, self.pictures.len() - 1);

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

    fn to_preview(&self) -> PreviewPicture {
        let src_prefix = match self.path.extension().unwrap().to_str() {
            Some("jpg") => "data:image/jpg;base64,",
            Some("png") => "data:image/png;base64,",
            Some("jpeg") => "data:image/jpg;base64,",
            _ => "",
        };

        PreviewPicture {
            picture: self.clone(),
            // thumbnail: String::new(),
            thumbnail: match self.thumbnail(256) {
                Some(s) => String::from(src_prefix) + &s,
                None => String::new(),
            },
        }
    }

    fn thumbnail(&self, max_px: u32) -> Option<String> {
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

        return Some(general_purpose::STANDARD_NO_PAD.encode(buffer.get_ref()));
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
