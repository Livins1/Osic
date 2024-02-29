/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// if we add new fields, give them default values when deserializing old state
use crossbeam::channel;
use egui::{Color32, Layout, TextBuffer, WidgetText};
use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;

use egui::{FontFamily, FontId, RichText, TextStyle};
use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};

use crate::data;
use crate::data::config::AppConfig;
use crate::data::monitor::Monitor;

// const PAGES: Vec<&str> = Vec["Library", "Options", "Modes", "Exit"];

const PAGES: &'static [&'static str] = &["Library", "Options", "Modes", "Exit"];

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrayMessage {
    SettingsShow,
    OnIconDoubleClick,
    OnIconClick,
    Exit,
}

#[derive(Debug, PartialEq)]
pub enum Pages {
    Library,
    Options,
    Modes,
    Exit,
}
impl Pages {
    fn find(page: &str) -> Pages {
        match page {
            "Library" => Pages::Library,
            "Options" => Pages::Options,
            "Modes" => Pages::Modes,
            "Exit" => Pages::Exit,
            _ => Pages::Library,
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
    page: Pages,
    monitors: Vec<Monitor>,

    selected_monitor: Monitor,
    selected_wp_path: String,
    _tray_start: bool,
    _tray_icon: TrayIcon<TrayMessage>,
    _tray_icon_inner: Arc<RwLock<TrayIconInner>>,
}
struct TrayIconInner {
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
        println!("New App Created!");

        Self {
            label: "Label Stuff".to_string(),
            page: Pages::Library,
            selected_wp_path: String::from(""),
            selected_monitor: monitors.first().unwrap().clone(),
            monitors: monitors,
            config: AppConfig::load_from_file(),
            _tray_start: false,
            _tray_icon: tray_icon,
            _tray_icon_inner: Arc::new(RwLock::new(TrayIconInner {
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

    fn tray_monitor(&mut self, ctx: egui::Context) {
        let tray = self._tray_icon_inner.clone();

        thread::spawn(move || {
            let receiver = tray.read().unwrap().tray_receiver.clone();
            while let Ok(message) = receiver.recv() {
                // let mut lock = this_share.lock().unwrap();
                let mut tray = tray.write().unwrap();
                match message {
                    TrayMessage::SettingsShow => {
                        tray.is_visible = true;
                    }
                    TrayMessage::Exit => {
                        tray.is_close = true;
                    }
                    TrayMessage::OnIconDoubleClick => {
                        tray.is_visible = !tray.is_visible;
                        ctx.request_repaint();
                    }
                    TrayMessage::OnIconClick => {}
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
    fn on_close_event(&mut self) -> bool {
        self.set_visible(false);
        // self._tray_icon_inner

        self.config.save_to_toml();

        self._tray_icon_inner.read().unwrap().is_close
        // self.is_close
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
            self.tray_monitor(ctx.clone());
            self._tray_start = true;
        }

        if self.get_close() {
            _frame.close();
        }

        _frame.set_visible(self.get_visible());

        egui::TopBottomPanel::top("TopPanel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::Grid::new("Monitors")
                    .spacing([5.0, 5.0])
                    .show(ui, |ui| {
                        for monitor in &self.monitors {
                            if ui
                                .add(egui::SelectableLabel::new(
                                    self.selected_monitor.device_id == monitor.device_id,
                                    &monitor.name,
                                ))
                                .clicked()
                            {
                                self.selected_monitor = monitor.clone()
                            }
                            // ui.selectable_label(
                            //     self.selected_monitor.device_id == monitor.device_id,
                            //     &monitor.name,
                            // );
                            // ui.end_row();
                        }
                    })
            })
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Add Path").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    println!("Get Path : {:?}", path.display().to_string());
                    self.config.add_wp_dirs(path.display().to_string());
                }
            }
            ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.style_mut().visuals.extreme_bg_color = Color32::BLACK;
                ui.visuals_mut().selection.bg_fill = Color32::BLACK;
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .max_width(300.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        ui.add_space(5.0);
                        ui.horizontal_wrapped(|ui| {
                            for wp_path in self.config.get_wp_dirs() {
                                ui.add_space(5.0);
                                let mut button = egui::Button::new(
                                    RichText::new(wp_path).text_style(TextStyle::Body),
                                )
                                .frame(false)
                                .wrap(true);

                                if self.selected_wp_path.eq(wp_path) {
                                    button = button.fill(Color32::from_white_alpha(10));
                                }

                                if ui.add(button).clicked() {
                                    println!("clicked path, {}", wp_path);
                                    self.selected_wp_path = wp_path.to_string();
                                }
                                ui.end_row();
                            }
                        })
                    })
            });

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
        resizable: true,
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
        eframe::run_native(
            "Osic-Windows",
            native_options,
            Box::new(|cc| Box::new(App::new(cc, tray_icon, tray_rx, monitors))),
        );
    }
}
