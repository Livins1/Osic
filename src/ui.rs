/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// if we add new fields, give them default values when deserializing old state
use crossbeam::channel;

use crossbeam::epoch::Pointable;
use egui::{
    util, vec2, Button, Color32, ColorImage, Image, ImageData, Layout, Margin, TextBuffer,
    TextureOptions, WidgetText,
};
use image::flat;
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, BorrowMut, Cow};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, panicking};
use std::time::Duration;
use windows::Win32::Foundation::NOERROR;

use egui::{FontFamily, FontId, RichText, TextStyle};
use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};

use crate::cache::{OsicMonitorSettings, OsicRecentImage};
use crate::data::config::AppConfig;
// use crate::data::monitor::Monitor;
use crate::data::{self, monitor};
use crate::selector::{self, OsicSlideSelector};
use crate::win32::Monitor;
use crate::win32::Win32API;
use crate::{cache, utils, win32};

// const PAGES: Vec<&str> = Vec["Library", "Options", "Modes", "Exit"];

// const PAGES: &'static [&'static str] = &["Library", "Options", "Modes", "Exit"];
const MODES: &'static [&'static str] = &["Picture", "SlidShow"];
const FITS: &'static [&'static str] = &["Fill", "Fit", "Stretch", "Tile", "Center", "Span"];
const INTERVAL: &'static [&'static str] =
    &["1 minute", "10 minutes", "30 minutes", "1 hour", "6 hour"];

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrayMessage {
    SettingsShow,
    OnIconDoubleClick,
    OnIconClick,
    Exit,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Fits {
    Fill = 4,
    Fit = 3,
    Stretch = 2,
    Tile = 1,
    Center = 0,
    Span = 5,
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Interval {
    OneMinute,
    TenMinutes,
    ThirtyMinutes,
    OneHour,
    SixHours,
}

impl Interval {
    fn find(i: &str) -> Interval {
        match i {
            "1 minute" => Interval::OneMinute,
            "10 minutes" => Interval::TenMinutes,
            "30 minutes" => Interval::ThirtyMinutes,
            "1 hour" => Interval::OneHour,
            "6 hour" => Interval::SixHours,
            _ => Interval::TenMinutes,
        }
    }

    fn seconds(i: &Interval) -> u64 {
        match i {
            Interval::OneMinute => 60,
            Interval::TenMinutes => 600,
            Interval::ThirtyMinutes => 60 * 30,
            Interval::OneHour => 60 * 60,
            Interval::SixHours => 60 * 60 * 6,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Interval::OneMinute => "1 minute",
            Interval::TenMinutes => "10 minutes",
            Interval::ThirtyMinutes => "30 minutes",
            Interval::OneHour => "1 hour",
            Interval::SixHours => "6 hours",
        }
    }
}

#[derive(Clone)]
pub struct MonitorWrapper {
    label: String,
    app_ctx: egui::Context,
    win32: Arc<win32::Win32API>,
    pub property: Monitor,
    pub album_path: Option<PathBuf>,
    pub mode: Modes,
    pub image: Option<PathBuf>,
    pub recent_images: Option<VecDeque<OsicRecentImage>>,
    pub fit: Fits,
    pub slide_interval: Interval,
    pub slide_time: u64,
    pub selector: OsicSlideSelector,
}

impl MonitorWrapper {
    fn new(monitor: Monitor, ctx: egui::Context, win32: Arc<Win32API>) -> Self {
        Self {
            label: monitor.name.clone(),
            app_ctx: ctx,
            win32: win32,
            property: monitor,
            mode: Modes::Picture,
            image: None,
            recent_images: None,
            fit: Fits::Fill,
            album_path: None,
            slide_interval: Interval::TenMinutes,
            slide_time: 0,
            selector: OsicSlideSelector::default(),
        }
    }

    fn from_cache(
        monitor: Monitor,
        ctx: egui::Context,
        win32: Arc<Win32API>,
        settings: OsicMonitorSettings,
    ) -> Self {
        let mut s = Self {
            label: monitor.name.clone(),
            app_ctx: ctx,

            win32: win32,
            property: monitor,
            album_path: settings.album_path,
            mode: settings.mode,
            image: settings.image,
            recent_images: None,
            fit: settings.fit,
            slide_interval: settings.slide_interval,
            slide_time: settings.slide_time,
            selector: settings.selector,
        };

        if let Some(images) = settings.recent_images {
            for img_path in images {
                if let Ok(img) = cache::load_image_cache(&img_path) {
                    let i = img.to_rgba8();
                    let t = utils::imgbuff_to_egui_imgdata(i);
                    let r = OsicRecentImage {
                        path: img_path,
                        thumbnail: t.clone(),
                        thumbnail_texture: s.app_ctx.load_texture("", t, TextureOptions::default()),
                    };
                    s.new_recent_image(r);
                }
            }
        }

        return s;
    }

    fn set_mode(&mut self, mode: Modes) {
        self.mode = mode;
    }

    fn set_fits(&mut self, fit: Fits) {
        let _ = self.win32.set_fit(fit as i32);
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
        if !self.image.is_none() && self.image.as_ref().unwrap().eq(&picture) {
            println!("Same Picture");
            return;
        };

        self.set_wallpaper(&picture);

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

    fn set_album(&mut self, path: PathBuf) {
        self.selector.set_album_path(path.clone());
        self.album_path = Some(path);
    }

    fn set_wallpaper(&self, path: &PathBuf) {
        let _ = self
            .win32
            .set_wallpaper(&self.property.device_id, path.to_str().unwrap());
    }

    fn set_slide_time(&mut self, time_stamp_sec: u64) {
        self.slide_time = time_stamp_sec;
    }

    fn set_slide_interval(&mut self, interval: Interval) {
        // self.slide_time = time_stamp_sec;
        self.slide_interval = interval;
    }

    fn ui_set_interval(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_width(ui.available_width() * 0.7);
            ui.label(RichText::new("Change picture every").color(Color32::WHITE));
        });

        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
            ui.set_width(ui.available_width() * 0.2);
            // ui.set_height(25.0);
            // ui.add_space(40.0);
            let m = &self.slide_interval;
            egui::ComboBox::from_id_source("interval_combobox")
                .width(120.0)
                .selected_text(m.as_str())
                .show_ui(ui, |ui| {
                    for i in INTERVAL {
                        ui.selectable_value(
                            &mut self.slide_interval,
                            Interval::find(i),
                            i.to_string(),
                        );
                    }
                })
        });
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
    tick: u64,
    tick_interval: u64,
    tick_status: bool,

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
        win32: Arc<Win32API>,
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
            .map(|monitor| match monitor.load_from_cache() {
                Ok(s) => MonitorWrapper::from_cache(monitor, ctx.clone(), win32.clone(), s),
                Err(_) => MonitorWrapper::new(monitor, ctx.clone(), win32.clone()),
            })
            .collect();

        Self {
            label: "Label Stuff".to_string(),
            selected_monitor: 0,
            monitors: monitor_wrappers,
            tick: utils::get_sys_time_in_secs(),
            tick_interval: 10,
            tick_status: false,
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

    // fn current_monitor_unmut(&mut self) -> &MonitorWrapper {
    //     return self.monitors.get(self.selected_monitor).unwrap();
    // }

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

    fn ui_get_floder(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_width(ui.available_width() * 0.7);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Choose a picture album for a slideshow").color(Color32::WHITE),
                );
                if self.current_monitor().selector.path.is_dir() {
                    // ui.label(
                    //     RichText::new(
                    //         "A picture background or slideshow background apply to single desktop.",
                    //     )
                    //     .font(FontId::proportional(13.0)),
                    // );
                    ui.label(
                        RichText::new(self.current_monitor().selector.path.to_str().unwrap())
                            .font(FontId::proportional(13.0)),
                    );
                    ui.add_space(ui.available_width() * 0.05);
                }
            })
        });

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_width(ui.available_width() * 0.2);
            let button = ui.add_sized([120.0, 20.0], egui::Button::new("Browse"));

            if button.clicked() {
                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                    self.current_monitor().set_album(p);
                }
            }
        });
    }

    fn ui_get_picture(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            ui.set_width(ui.available_width() * 0.7);
            ui.label(RichText::new("Choose a photo").color(Color32::WHITE));
        });

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_width(ui.available_width() * 0.2);
            let button = ui.add_sized([120.0, 20.0], egui::Button::new("Browse photos"));

            if button.clicked() {
                if let Some(p) = rfd::FileDialog::new()
                    .add_filter("image", &["png", "jpg", "jpeg"])
                    .pick_file()
                {
                    // monitor.set_picture(p);
                    self.current_monitor().set_picture(p);
                }
            }
        });
    }

    // Some errors with egui,  waiting fix
    // https://github.com/emilk/egui/issues/3972
    fn reapint_tick(&self, ctx: &egui::Context) {
        let interval = self.tick_interval;
        let ctx = ctx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(interval));
            ctx.request_repaint();
            println!("Timer Tick!");
        });
    }

    fn slide_show_active(&mut self) {
        let c_stamp = utils::get_sys_time_in_secs();
        for monitor in &mut self.monitors {
            if monitor.mode == Modes::SlidShow {
                println!("Timer is {}", &c_stamp);
                if c_stamp > monitor.slide_time + Interval::seconds(&monitor.slide_interval) {
                    // println!("SLide !  interval: {}", monitor.slide_interval as u64);
                    monitor.set_slide_time(c_stamp);
                }
            }
        }
    }
}

impl eframe::App for App {
    // Old version!
    // fn on_close_event(&mut self) -> bool {
    //     self.set_visible(false);
    //     // self._tray_icon_inner

    //     // self.is_close
    // }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.set_visible(false);

        self.config.save_to_toml();

        for m in self.monitors.clone() {
            let _ = cache::write_monitor_settings(m.into());
        }

        // self._tray_icon_inner.read().unwrap().is_close
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self._tray_start {
            println!("Start Tray Event Monitor");
            self.tray_monitor(ctx);
            self._tray_start = true;
        }
        let update_tick = utils::get_sys_time_in_secs();

        if !self.tick_status {
            self.reapint_tick(ctx);
            self.tick_status = true;
        }

        if update_tick > self.tick + self.tick_interval {
            self.tick = update_tick;
            self.slide_show_active();
            // self.reapint_tick(ctx);
        };

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
                // .min_row_height(40.0)
                .spacing([15.0, 15.0])
                .show(ui, |ui| {
                    // let monitor = self.current_monitor_unmut();
                    ui.style_mut().spacing.button_padding = vec2(12.0, 5.0);
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
                            // ui.label(RichText::new("A picture background or slideshow background apply to single desktop.").font(FontId::proportional(13.0)));
                            ui.add_space(ui.available_width() * 0.05);
                        });

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                            ui.set_width(ui.available_width() * 0.2);
                            let m = &self.current_monitor().mode;
                            egui::ComboBox::from_id_source("modes")
                                .width(120.0)
                                .selected_text(format!("{m:?}"))
                                .show_ui(ui, |ui| {
                                    for mode in MODES {
                                        ui.selectable_value(
                                            &mut self.current_monitor().mode,
                                            Modes::find(mode),
                                            mode.to_string(),
                                        );
                                    }
                                });
                        });
                        ui.add_space(ui.available_width() * 0.02);
                    });
                    ui.end_row();
                    // ui.add_space(50.0);
                    // ui.end_row();
                    // Latesd photos.

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                        ui.add_space(ui.available_width() * 0.02);
                        match self.current_monitor().mode {
                            Modes::Picture => {
                                self.ui_get_picture(ui);
                            }
                            Modes::SlidShow => {
                                self.ui_get_floder(ui);
                            }
                        }
                        ui.add_space(ui.available_width() * 0.02);
                    });
                    ui.end_row();

                    if let Some(images) = &self.current_monitor().recent_images {
                        // ui.set_height(0.0);
                        // ui.set_row_height(50.0);

                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.add_space(ui.available_width() * 0.02);
                            ui.vertical(|ui| {
                                ui.label(RichText::new("Recent images").color(Color32::WHITE));
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

                            ui.add_space(ui.available_width() * 0.02);
                        });

                        ui.end_row();
                    }
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

                        // let fit = monitor.fit.borrow_mut();
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            ui.set_width(ui.available_width() * 0.2);

                            let fit = &self.current_monitor().fit;

                            egui::ComboBox::from_label("")
                                .width(120.0)
                                .selected_text(format!("{fit:?}"))
                                .show_ui(ui, |ui| {
                                    for f in FITS {
                                        if ui
                                            .selectable_value(
                                                &mut self.current_monitor().fit,
                                                Fits::find(f),
                                                f.to_string(),
                                            )
                                            .clicked()
                                        {
                                            // println!("{:?}", fit);
                                            self.current_monitor().set_fits(Fits::find(f));
                                        };
                                    }
                                });
                        });
                        ui.add_space(ui.available_width() * 0.02);
                    });

                    // ui.horizontal(|ui| {});
                    ui.end_row();

                    match self.current_monitor().mode {
                        Modes::Picture => {}
                        Modes::SlidShow => {
                            ui.horizontal(|ui| {
                                ui.add_space(ui.available_width() * 0.02);
                                self.current_monitor().ui_set_interval(ui);
                                ui.add_space(ui.available_width() * 0.02);
                            });
                            ui.end_row();
                            ui.horizontal(|ui| {
                                ui.add_space(ui.available_width() * 0.02);
                                self.current_monitor().selector.ui(ui);
                                ui.add_space(ui.available_width() * 0.02);
                            });
                            ui.end_row();
                        }
                    }
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

    let win32 = Win32API::new();

    if let Ok(monitors) = win32.get_monitor_device_path() {
        let _ = eframe::run_native(
            "Osic-Windows",
            native_options,
            Box::new(|cc| Box::new(App::new(cc, tray_icon, tray_rx, monitors, Arc::new(win32)))),
        );
    }
}
