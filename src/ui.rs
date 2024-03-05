/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// if we add new fields, give them default values when deserializing old state
use crossbeam::channel;
use crossbeam::epoch::Pointable;
use egui::{
    util, vec2, Button, Color32, ColorImage, Image, ImageData, Layout, Margin, TextBuffer,
    TextureOptions, WidgetText,
};
use image::flat;
use std::borrow::{Borrow, BorrowMut, Cow};
use std::collections::VecDeque;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::str::Bytes;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, panicking};

use egui::{FontFamily, FontId, RichText, TextStyle};
use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};

use crate::cache::OsicRecentImage;
use crate::data::config::AppConfig;
use crate::data::monitor::Monitor;
use crate::data::{self, monitor};
use crate::{cache, utils};

// const PAGES: Vec<&str> = Vec["Library", "Options", "Modes", "Exit"];

// const PAGES: &'static [&'static str] = &["Library", "Options", "Modes", "Exit"];
const MODES: &'static [&'static str] = &["Picture", "SlidShow"];
const FITS: &'static [&'static str] = &["Fill", "Fit", "Stretch", "Tile", "Center", "Span"];

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrayMessage {
    SettingsShow,
    OnIconDoubleClick,
    OnIconClick,
    Exit,
}

// #[derive(Debug, PartialEq)]
// pub enum Pages {
//     Library,
//     Options,
//     Modes,
//     Exit,
// }
// impl Pages {
//     fn find(page: &str) -> Pages {
//         match page {
//             "Library" => Pages::Library,
//             "Options" => Pages::Options,
//             "Modes" => Pages::Modes,
//             "Exit" => Pages::Exit,
//             _ => Pages::Library,
//         }
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub enum Modes {
    Picture,
    SlidShow,
}

impl Modes {
    fn find(mode: &str) -> Modes {
        match mode {
            "Picture" => Modes::Picture,
            "SlidShow" => Modes::SlidShow,
            _ => Modes::Picture,
        }
    }
}

// const FITS: &'static [&'static str] = &["Fill", "Fit", "Stretch", "Tile", "Center", "Span"];

#[derive(Debug, PartialEq, Clone)]
pub enum Fits {
    Fill,
    Fit,
    Stretch,
    Tile,
    Center,
    Span,
}
impl Fits {
    fn find(mode: &str) -> Fits {
        match mode {
            "Fill" => Fits::Fill,
            "Fit" => Fits::Fit,
            "Stretch" => Fits::Stretch,
            "Tile" => Fits::Tile,
            "Center" => Fits::Center,
            "Span" => Fits::Span,
            _ => Fits::Fill,
        }
    }
}

#[derive(Clone)]
pub struct MonitorWrapper {
    label: String,
    property: Monitor,
    app_ctx: egui::Context,
    mode: Modes,
    image: Option<PathBuf>,
    // recent_image: Option<Vec<PathBuf>>,
    recent_images: Option<VecDeque<OsicRecentImage>>,
    fit: Fits,
    // image_buffer: Option<ImageData>,
}

impl MonitorWrapper {
    fn new(monitor: Monitor, ctx: egui::Context) -> Self {
        Self {
            label: monitor.name.clone(),
            app_ctx: ctx,
            property: monitor,
            mode: Modes::Picture,
            image: None,
            recent_images: None,
            fit: Fits::Fill,
        }
    }

    fn set_mode(&mut self, mode: Modes) {
        self.mode = mode;
    }

    fn find_recent_image(&self, path: &PathBuf) -> Option<(usize, &OsicRecentImage)> {
        if let Some(images) = &self.recent_images {
            for (index, image) in images.into_iter().enumerate() {
                if image.path.to_str().unwrap() == path.to_str().unwrap() {
                    return Some((index, &image));
                }
            }
        };
        None
    }

    fn remove_recent_image(&mut self, path: &PathBuf) {
        if let Some((index, _)) = self.find_recent_image(path) {
            self.recent_images.as_mut().unwrap().remove(index);
        }
    }

    fn new_recent_image(&mut self, i: OsicRecentImage) {
        self.remove_recent_image(&i.path);
        if let Some(images) = &mut self.recent_images {
            if images.len() > 4 {
                println!("images len :{}", images.len());
                for _ in 0..images.len() - 4 {
                    images.pop_back();
                }
            }
            images.push_front(i);
        } else {
            self.recent_images = Some(VecDeque::from([i]));
        }
    }

    fn set_picture(&mut self, picture: PathBuf) {
        self.image = Some(picture.clone());

        if let Ok(img) = cache::load_image_cache(&picture) {
            let i = img.to_rgba8();
            let t = utils::imgbuff_to_egui_imgdata(i);
            let r = OsicRecentImage {
                path: picture,
                thumbnail: t.clone(),
                thumbnail_texture: self.app_ctx.load_texture("", t, TextureOptions::default()),
            };
            self.new_recent_image(r);
        } else {
            let i = utils::gen_gallery(&picture, 100);
            if let Ok(_) = cache::write_image_cache(&picture, &i) {
                let t = utils::imgbuff_to_egui_imgdata(i);
                self.new_recent_image(OsicRecentImage {
                    path: picture,
                    thumbnail: t.clone(),
                    thumbnail_texture: self.app_ctx.load_texture("", t, TextureOptions::default()),
                });
            }
        }
    }
}

fn configure_text_styles(ctx: &egui::Context) {
    use FontFamily::{Monospace, Proportional};

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Proportional)),
        (TextStyle::Body, FontId::new(16.0, Proportional)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(12.0, Proportional)),
        (TextStyle::Small, FontId::new(8.0, Proportional)),
        (
            TextStyle::Name("Page".into()),
            FontId::new(23.0, Proportional),
        ),
        (
            TextStyle::Name("Big Button".into()),
            FontId::new(20.0, Proportional),
        ),
    ]
    .into();
    ctx.set_style(style);
}

// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)]
pub struct App {
    // Example stuff:
    label: String,
    config: AppConfig,
    monitors: Vec<MonitorWrapper>,

    selected_monitor: usize,
    _tray_start: bool,
    _tray_icon: TrayIcon<TrayMessage>,
    _tray_icon_inner: Arc<RwLock<TrayIconInner>>,
}
struct TrayIconInner {
    ctx: egui::Context,
    is_close: bool,
    is_visible: bool,
    tray_receiver: channel::Receiver<TrayMessage>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        tray_icon: TrayIcon<TrayMessage>,
        tray_receiver: channel::Receiver<TrayMessage>,
        monitors: Vec<Monitor>,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

        // }
        configure_text_styles(&cc.egui_ctx);

        egui_extras::install_image_loaders(&cc.egui_ctx);
        let ctx = cc.egui_ctx.clone();

        println!("New App Created!");
        let monitor_wrappers: Vec<MonitorWrapper> = monitors
            .into_iter()
            .map(|monitor| MonitorWrapper::new(monitor, ctx.clone()))
            .collect();

        Self {
            label: "Label Stuff".to_string(),
            selected_monitor: 0,
            monitors: monitor_wrappers,
            config: AppConfig::load_from_file(),
            _tray_start: false,
            _tray_icon: tray_icon,
            _tray_icon_inner: Arc::new(RwLock::new(TrayIconInner {
                ctx: ctx,
                is_close: false,
                is_visible: true,
                tray_receiver: tray_receiver,
            })),
        }
    }
    fn set_visible(&mut self, visible: bool) {
        // self._tray_icon_inner.read().unwrap().is_visible = visible;
        self._tray_icon_inner.write().unwrap().is_visible = visible;
        // self.is_visible = visible
    }
    fn set_close(&mut self, close: bool) {
        self._tray_icon_inner.write().unwrap().is_close = close;
        // self.is_close = close
    }
    fn get_visible(&self) -> bool {
        self._tray_icon_inner.read().unwrap().is_visible
    }
    fn get_close(&self) -> bool {
        self._tray_icon_inner.read().unwrap().is_close
    }
    fn current_monitor(&mut self) -> &mut MonitorWrapper {
        return self.monitors.get_mut(self.selected_monitor).unwrap();
    }
    fn current_monitor_unmut(&mut self) -> &MonitorWrapper {
        return self.monitors.get(self.selected_monitor).unwrap();
    }

    fn tray_monitor(&mut self, ctx: &egui::Context) {
        let tray = self._tray_icon_inner.clone();
        let v_id = ctx.viewport_id();
        thread::spawn(move || {
            let receiver = tray.read().unwrap().tray_receiver.clone();
            while let Ok(message) = receiver.recv() {
                // let mut lock = this_share.lock().unwrap();
                let mut tray = tray.write().unwrap();
                match message {
                    TrayMessage::SettingsShow => {
                        tray.is_visible = true;
                        tray.ctx
                            .send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        // tray.ctx
                        //     .send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                        // tray.ctx.send_viewport_cmd_to(v_id, egui::ViewportCommand::Minimized(false));

                        tray.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        tray.ctx.request_repaint();
                    }
                    TrayMessage::Exit => {
                        // tray.is_close = true;
                        // ctx.send_viewport_cmd_to(v_id, egui::ViewportCommand::Close);
                        // ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        // ctx.request_repaint_of(v_id);
                        tray.ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        tray.ctx.request_repaint();
                        println!("exit!");
                    }
                    TrayMessage::OnIconDoubleClick => {
                        tray.is_visible = !tray.is_visible;
                        println!("DoubleClick: {:?}", tray.is_visible);
                        // ctx.send_viewport_cmd_to(
                        //     v_id,
                        //     egui::ViewportCommand::Maximized(tray.is_visible),
                        // );
                        // tray.ctx
                        //     .send_viewport_cmd(egui::ViewportCommand::Visible(tray.is_visible));

                        if tray.is_visible {
                            // tray.ctx
                            //     .send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                            tray.ctx
                                .send_viewport_cmd(egui::ViewportCommand::Visible(tray.is_visible));
                        } else {
                            // tray.ctx
                            //     .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            tray.ctx
                                .send_viewport_cmd(egui::ViewportCommand::Visible(tray.is_visible));
                        }
                        tray.ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                        tray.ctx.request_repaint();

                        // ctx.request_repaint_of(v_id);
                        // ctx.request_repaint();
                    }
                    TrayMessage::OnIconClick => {

                        // ctx.send_viewport_cmd(egui::ViewportCommand::);
                    }
                }
            }
        });
    }

    // fn tray_message(&mut self, _frame: &mut eframe::Frame) {
    //     // let _ = self._tray_icon;

    //     while let Ok(message) = self.tray_receiver.recv() {}
    // }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    // Old version!
    // fn on_close_event(&mut self) -> bool {
    //     self.set_visible(false);
    //     // self._tray_icon_inner

    //     // self.is_close
    // }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.set_visible(false);

        self.config.save_to_toml();

        // self._tray_icon_inner.read().unwrap().is_close
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let Self {  } = self;
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        // tray_polling
        // self.tray_message(_frame);
        if !self._tray_start {
            println!("Start Tray Event Monitor");
            self.tray_monitor(ctx);
            self._tray_start = true;
        }

        // if self.get_close() {
        //     ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        // }

        // _frame.set_visible(self.get_visible())

        // ctx.send_viewport_cmd(egui::ViewportCommand::Visible(self.get_visible()));

        egui::TopBottomPanel::top("TopPanel")
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    egui::Grid::new("Monitors")
                        .spacing([5.0, 5.0])
                        .show(ui, |ui| {
                            for (index, monitor) in self.monitors.iter().enumerate() {
                                if ui
                                    .add(egui::SelectableLabel::new(
                                        &self
                                            .monitors
                                            .get(self.selected_monitor)
                                            .unwrap()
                                            .property
                                            .device_id
                                            == &monitor.property.device_id,
                                        &monitor.property.name,
                                    ))
                                    .clicked()
                                {
                                    self.selected_monitor = index
                                }
                            }
                        })
                })
            });

        egui::TopBottomPanel::bottom("BottomPanel")
            .show_separator_line(false)
            .min_height(100.0)
            .max_height(100.0)
            // .max_height(200.0)
            .show(ctx, |ui| {
                ui.add_space(50.0);
                ui.horizontal(|ui| {
                    ui.set_max_height(100.0);
                    ui.style_mut().spacing.button_padding = vec2(1.0, 8.0);
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::new(15.0, FontFamily::Proportional),
                    );
                    ui.columns(2, |cols| {
                        cols[0].vertical_centered_justified(|ui| {
                            ui.set_width(150.0);
                            ui.set_height(100.0);
                            let cencel_button =
                                ui.add_sized([20.0, 20.0], egui::Button::new("Save & Minimize"));

                            if cencel_button.clicked() {
                                println!("Save& Minimize");
                            }
                        });
                        cols[1].vertical_centered_justified(|ui| {
                            ui.set_width(150.0);
                            ui.set_height(100.0);
                            let cencel_button =
                                ui.add_sized([20.0, 20.0], egui::Button::new("Minimize"));

                            if cencel_button.clicked() {
                                println!("Minimize");
                            }
                        });
                    });
                });
                // ui.add_space(ui.wi);

                // ui.style_mut().text_styles.insert(
                //     egui::TextStyle::Button,
                //     egui::FontId::new(15.0, FontFamily::Proportional),
                // );
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // ui.spacing_mut().item_spacing.y = 10.0;

            egui::Grid::new("Monitors_Mode")
                .num_columns(1)
                .min_row_height(40.0)
                // .spacing([10.0, 10.0])
                .show(ui, |ui| {
                    let monitor = self.current_monitor();
                    ui.style_mut().spacing.button_padding = vec2(12.0, 6.0);
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::new(14.0, FontFamily::Proportional),
                    );
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() * 0.02);
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.set_width(ui.available_width() * 0.7);
                            ui.label(
                                RichText::new("Personalize your background").color(Color32::WHITE),
                            );
                            ui.label(RichText::new("A picture background or slideshow background apply to single desktop.").font(FontId::proportional(14.0)));
                        });

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_width(ui.available_width() * 0.2);
                            let m = monitor.mode.borrow_mut();
                            egui::ComboBox::from_id_source("modes")
                                .width(120.0)
                                .selected_text(format!("{m:?}"))
                                .show_ui(ui, |ui| {
                                    for mode in MODES {
                                        ui.selectable_value(m, Modes::find(mode), mode.to_string());
                                    }
                                });
                        });
                        ui.add_space(ui.available_width() * 0.02);
                    });
                    ui.end_row();
                    // ui.add_space(50.0);
                    // ui.end_row();
                    // Latesd photos.
                    if let Some(images) = &monitor.recent_images {
                        ui.horizontal(|ui| {
                            // ui.set_height(90.0);
                            ui.add_space(ui.available_width() * 0.02);
                            ui.with_layout(egui::Layout::top_down(egui::Align::TOP), |ui| {
                                ui.set_width(ui.available_width() * 0.7);
                                ui.label(
                                    RichText::new("Recent images")
                                        .color(Color32::WHITE)
                                );
                                ui.horizontal(|ui| {
                                        for image in images {
                                            ui.add(
                                                egui::Image::new(&image.thumbnail_texture)
                                                    .rounding(5.0)
                                                    .max_size([75.0, 75.0].into()),
                                            );
                                        }
                                });

                            });
                        });
                        ui.end_row();
                        // ui.add_space(50.0);
                    }

                    ui.add_space(40.0);
                    ui.end_row();
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() * 0.02);
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.set_width(ui.available_width() * 0.7);
                            ui.label(RichText::new("Choose a photo").color(Color32::WHITE));
                        });

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_width(ui.available_width() * 0.2);
                            let button =
                                ui.add_sized([120.0, 20.0], egui::Button::new("Browse photos"));

                            if button.clicked() {
                                if let Some(p) = rfd::FileDialog::new()
                                    .add_filter("image", &["png", "jpg", "jpeg"])
                                    .pick_file()
                                {
                                    monitor.set_picture(p);
                                }
                            }
                        });
                        ui.add_space(ui.available_width() * 0.02);
                    });
                    ui.end_row();
                    // ui.add_space(25.0);
                    // ui.end_row();
                    // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    //     // egui::Button::new("Browse photos").min_size([120.0, 20])
                    //     let button =
                    //         ui.add_sized([120.0, 20.0], egui::Button::new("Browse photos"));

                    //     if button.clicked() {
                    //         if let Some(p) = rfd::FileDialog::new()
                    //             .add_filter("image", &["png", "jpg", "jpeg"])
                    //             .pick_file()
                    //         {
                    //             // println!("PicturePath: {:?}", p.display().to_string());
                    //             // self.current_monitor().set_picture(p);
                    //             monitor.set_picture(p);
                    //         }
                    //     }
                    // });



                    // Choose fit
                    ui.horizontal(|ui| {
                        ui.add_space(ui.available_width() * 0.02);
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.set_width(ui.available_width() * 0.7);
                            ui.label(
                                RichText::new("Choose a fit for your desktop image")
                                    .color(Color32::WHITE),
                            );
                        });

                        let fit = monitor.fit.borrow_mut();
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.set_width(ui.available_width() * 0.2);

                            egui::ComboBox::from_label("")
                                .width(120.0)
                                .selected_text(format!("{fit:?}"))
                                .show_ui(ui, |ui| {
                                    for f in FITS {
                                        ui.selectable_value(fit, Fits::find(f), f.to_string());
                                    }
                                });
                        });
                        ui.add_space(ui.available_width() * 0.02);
                    });

                    // ui.horizontal(|ui| {});
                    ui.end_row();
                });

            // ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
            //     ui.horizontal(|ui| {
            //         ui.spacing_mut().item_spacing.x = 5.0;
            //         ui.label("Recent images :");
            //         if let Some(images) = &monitor.recent_images {
            //             for image in images {
            //                 // ui.image(&image.thumbnail_texture).;
            //                 ui.add(
            //                     egui::Image::new(&image.thumbnail_texture)
            //                         .rounding(5.0)
            //                         .max_size([75.0, 75.0].into()),
            //                 );
            //             }
            //         }
            //     });
            // });

            // if ui.button("Add Path").clicked() {
            //     if let Some(path) = rfd::FileDialog::new().pick_folder() {
            //         println!("Get Path : {:?}", path.display().to_string());
            //         self.config.add_wp_dirs(path.display().to_string());
            //     }
            // }
            // ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
            //     ui.style_mut().visuals.extreme_bg_color = Color32::BLACK;
            //     ui.visuals_mut().selection.bg_fill = Color32::BLACK;
            //     egui::ScrollArea::vertical()
            //         .max_height(200.0)
            //         .max_width(300.0)
            //         .auto_shrink([false; 2])
            //         .show(ui, |ui| {
            //             ui.add_space(5.0);
            //             ui.horizontal_wrapped(|ui| {
            //                 for wp_path in self.config.get_wp_dirs() {
            //                     ui.add_space(5.0);
            //                     let mut button = egui::Button::new(
            //                         RichText::new(wp_path).text_style(TextStyle::Body),
            //                     )
            //                     .frame(false)
            //                     .wrap(true);

            //                     if self.selected_wp_path.eq(wp_path) {
            //                         button = button.fill(Color32::from_white_alpha(10));
            //                     }

            //                     if ui.add(button).clicked() {
            //                         println!("clicked path, {}", wp_path);
            //                         self.selected_wp_path = wp_path.to_string();
            //                     }
            //                     ui.end_row();
            //                 }
            //             })
            //         })
            // });

            // .show(ui, |ui| {
            //     // ui.painter()
            //     //     .rect_filled(ui.available_rect_before_wrap(), 10.0, Color32::BLACK);
            //     // ui.painter().rect_stroke(
            //     //     ui.max_rect(),
            //     //     5.0,
            //     //     ui.visuals().selection.stroke,
            //     // );
            // });
        });

        // ctx.request_repaint_after(duration)

        // ctx.request_repaint();

        // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("File", |ui| {
        //             if ui.button("Quit").clicked() {
        //                 _frame.set_visible(false);
        //             }
        //         });

        //         if ui.button("Minimize").clicked() {
        //             _frame.set_visible(false);
        //         }
        //     });
        // });
        // custom_window_frame(ctx, _frame, "egui with custom frame", |ui| {
        //     ui.label("This is just the contents of the window");
        //     ui.horizontal(|ui| {
        //         ui.label("egui theme:");
        //         egui::widgets::global_dark_light_mode_buttons(ui);
        //     });
        // });

        // egui::SidePanel::left("side_panel").show(ctx, |ui| {
        //     ui.heading("Side Panel");

        //     ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        //         ui.horizontal(|ui| {
        //             ui.spacing_mut().item_spacing.x = 0.0;
        //             ui.label("powered by ");
        //             ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        //             ui.label(" and ");
        //             ui.hyperlink_to(
        //                 "eframe",
        //                 "https://github.com/emilk/egui/tree/master/crates/eframe",
        //             );
        //             ui.label(".");
        //         });
        //     });
        // });

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     // The central panel the region left after adding TopPanel's and SidePanel's

        //     ui.heading("eframe template");
        //     ui.hyperlink("https://github.com/emilk/eframe_template");
        //     ui.add(egui::github_link_file!(
        //         "https://github.com/emilk/eframe_template/blob/master/",
        //         "Source code."
        //     ));
        //     egui::warn_if_debug_build(ui);
        // });

        // if false {
        //     egui::Window::new("Window").show(ctx, |ui| {
        //         ui.label("Windows can be moved by dragging them.");
        //         ui.label("They are automatically sized based on contents.");
        //         ui.label("You can turn on resizing and scrolling if you like.");
        //         ui.label("You would normally choose either panels OR windows.");
        //     });
        // }
    }
}

pub fn ui_init() {
    // let native_options = eframe::NativeOptions::default();
    let native_options = eframe::NativeOptions {
        // persist_window:true,
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size([580.0, 480.0])
            .with_max_inner_size([580.0, 480.0])
            .with_resizable(true)
            .with_maximize_button(false),

        ..Default::default()
    };

    let (tray_tx, tray_rx) = channel::unbounded();

    let icon = include_bytes!("../resource/icon/icon3.ico");

    let tray_icon = TrayIconBuilder::new()
        .sender_crossbeam(tray_tx)
        .icon_from_buffer(icon)
        .tooltip("Osic")
        .on_click(TrayMessage::OnIconClick)
        .on_double_click(TrayMessage::OnIconDoubleClick)
        .menu(
            MenuBuilder::new()
                .item("Settings", TrayMessage::SettingsShow)
                .item("Exit", TrayMessage::Exit),
        )
        .build()
        .unwrap();

    if let Ok(monitors) = data::monitor::get_monitor_device_path() {
        let _ = eframe::run_native(
            "Osic-Windows",
            native_options,
            Box::new(|cc| Box::new(App::new(cc, tray_icon, tray_rx, monitors))),
        );
    }
}
