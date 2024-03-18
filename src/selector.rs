use egui::{pos2, Color32, Rect, RichText};
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
    // the ratio of monitor
    pub ratio_value: f32,
    pub ratio: bool,
    pub ratio_range: f32,
    ratio_range_ui: f32,
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
    fn set_ratio_range(&mut self, range: f32) {
        self.ratio_range = range;
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
        if !self.ratio || self.pictures.is_none() {
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

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_width(ui.available_width() * 0.3);
            ui.label(RichText::new("Selector settings").color(Color32::WHITE));
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.set_width(ui.available_width() * 0.7);
            ui.set_height(25.0);
            ui.add_space(40.0);
            // let button = ui.add_sized([120.0, 20.0], egui::Button::new("Browse photos"));

            ui.columns(3, |cols| {
                cols[0].vertical_centered_justified(|ui| {
                    ui.set_width(75.0);
                    // ui.set_height(100.0);
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| {
                            if ui
                                .checkbox(&mut self.shuffle, "Shuffle")
                                .on_hover_text("Shuffle the picture order")
                                .clicked()
                            {
                                println!("Shuffle clicked")
                            };
                        },
                    );

                    // ui.add_space(25.0);
                });
                cols[1].vertical_centered_justified(|ui| {
                    ui.set_width(75.0);
                    // ui.set_height(100.0);
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::TopDown),
                        |ui| ui.checkbox(&mut self.ratio, "Ratio"),
                    );
                    // ui.add_space(25.0);
                });
                if self.ratio {
                    cols[2].vertical_centered_justified(|ui| {
                        // ui.set_width(50.0);
                        let i = ui.add(
                            egui::DragValue::new(&mut self.ratio_range_ui)
                                .clamp_range(0.0..=0.1)
                                .custom_formatter(|n, _| format!("{:.2}", n))
                                .speed(0.002),
                        );
                        if i.lost_focus() || i.drag_released() {
                            if self.ratio_range_ui != self.ratio_range {
                                self.set_ratio_range(self.ratio_range_ui);
                            }
                        }
                        // if i.drag_released() {
                        //     if self.ratio_range_ui != self.ratio_range {
                        //         self.set_ratio_range(self.ratio_range_ui);
                        //     }
                        // }

                        // ui.set_height(100.0);
                    });
                }
            });
            // if button.clicked() {
            //     if let Some(p) = rfd::FileDialog::new()
            //         .add_filter("image", &["png", "jpg", "jpeg"])
            //         .pick_file()
            //     {
            //         // monitor.set_picture(p);
            //         // self.current_monitor().set_picture(p);
            //     }
            // }
        });
    }
}
