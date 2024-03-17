use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::{
    alloc::System,
    borrow::{Borrow, BorrowMut},
    clone,
    path::PathBuf,
    time::SystemTime,
};

fn is_image_file(path: &PathBuf) -> bool {
    match ImageFormat::from_path(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OsicImageWrapper {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

impl OsicImageWrapper {
    pub fn new(path: PathBuf, width: u32, height: u32) -> Self {
        // let ratio = width as f32 / height as f32;
        OsicImageWrapper {
            path,
            width,
            height,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OsicSlideSelector {
    pub path: PathBuf,
    pub shuffle: bool,
    pub ratio_value: f32,
    pub ratio: bool,
    pub ratio_range: f32,
    pictures: Option<Vec<OsicImageWrapper>>,
    ratio_pool: Vec<usize>,
    wallpaper_index: usize,
}

impl OsicSlideSelector {
    fn set_shuffle(&mut self, need_shuffle: bool) {
        self.shuffle = need_shuffle;
    }

    fn set_ratio(&mut self, keep_ratio: bool) {
        self.ratio = keep_ratio;
        let _ = self.refresh_ratio_pool();
    }

    pub fn set_album_path(&mut self, p: PathBuf) {
        self.new_settings(
            p,
            self.shuffle,
            self.ratio,
            self.ratio_value,
            self.ratio_range,
        );
    }

    pub fn new_settings(
        &mut self,
        path: PathBuf,
        shuffle: bool,
        ratio: bool,
        ratio_value: f32,
        ratio_range: f32,
    ) {
        if !self.path.eq(&path) {
            self.path = path;
            self.fetch_picture();
            self.wallpaper_index = 0;
        }

        self.shuffle = shuffle;
        self.ratio = ratio;

        self.ratio_value = ratio_value;
        self.ratio_range = ratio_range;
        self.refresh_ratio_pool();
    }

    fn get_picture(&self, index: usize) -> Option<OsicImageWrapper> {
        if let Some(p) = &self.pictures {
            Some(p.get(index).unwrap().clone())
        } else {
            None
        }
    }

    fn sequence_ratio_picture(&mut self) -> Option<OsicImageWrapper> {
        let max = self.ratio_pool.len();
        let mut next = self.wallpaper_index + 1;

        if next > max - 1 {
            next = 0;
        }

        self.wallpaper_index = next;
        let index = self.ratio_pool.get(next).unwrap();
        return self.pictures.as_ref().unwrap().get(*index).cloned();
    }

    fn sequence_picture(&mut self) -> Option<OsicImageWrapper> {
        let max = self.pictures.as_ref().unwrap().len();
        let mut next = self.wallpaper_index + 1;

        if next > max - 1 {
            next = 0;
        }

        self.wallpaper_index = next;
        // let index = self.ratio_pool.get(next).unwrap();
        return self.pictures.as_ref().unwrap().get(next).cloned();
    }

    fn one(&mut self) -> Option<OsicImageWrapper> {
        if let Some(p) = &self.pictures {
            if p.is_empty() {
                return None;
            };

            if self.shuffle {
                if self.ratio {
                    let i = fastrand::usize(..self.ratio_pool.len());
                    let index = self.ratio_pool.get(i).unwrap();
                    return self.get_picture(*index);
                }
                let index = fastrand::usize(..self.pictures.as_ref().unwrap().len());
                return self.get_picture(index);
            }

            if self.ratio {
                return self.sequence_ratio_picture();
            }

            return self.sequence_picture();
        } else {
            return None;
        };
    }

    fn ratio_check(image: &OsicImageWrapper, ratio: f32, range: f32) -> bool {
        let image_ratio = image.width as f32 / image.height as f32;
        image_ratio >= ratio - range && image_ratio <= ratio + range
    }

    fn refresh_ratio_pool(&mut self) -> usize {
        if !self.ratio {
            return 0;
        };
        self.ratio_pool.clear();

        if let Some(ps) = &self.pictures {
            return ps
                .into_iter()
                .enumerate()
                .filter(|(index, p)| {
                    OsicSlideSelector::ratio_check(p, self.ratio_value, self.ratio_range)
                })
                .map(|(index, _)| self.ratio_pool.push(index))
                .count();
        }

        return self.ratio_pool.len();
    }

    fn add_picture(&mut self, p: OsicImageWrapper) -> usize {
        if let Some(ref mut b) = self.pictures {
            b.push(p);
            return b.len();
        } else {
            self.pictures = Some(vec![p]);
            return 1;
        };
    }

    fn fetch_picture(&mut self) -> usize {
        let mut n = 0;
        if let Ok(entries) = std::fs::read_dir(&self.path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if is_image_file(&entry.path()) {
                        if let Ok(size) = imagesize::size(&entry.path()) {
                            n = self.add_picture(OsicImageWrapper::new(
                                entry.path(),
                                size.width as u32,
                                size.height as u32,
                            ));
                        }
                    };
                };
            }
        };
        return n;
    }
}
