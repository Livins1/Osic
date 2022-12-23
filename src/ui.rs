/// We derive Deserialize/Serialize so we can persist app state on shutdown.
// if we add new fields, give them default values when deserializing old state
use crossbeam::channel;
use std::borrow::{Borrow, BorrowMut};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use windows::Win32::UI::Shell::SHARE_ROLE_READER;

use trayicon::{MenuBuilder, TrayIcon, TrayIconBuilder};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrayMessage {
    SettingsShow,
    OnIconDoubleClick,
    OnIconClick,
    Exit,
}

// #[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)]
pub struct App {
    // Example stuff:
    label: String,
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
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

        // }

        Self {
            label: "Label Stuff".to_string(),
            _tray_start: false,
            // is_close: false,
            // is_visible: false,
            _tray_icon: tray_icon,
            _tray_icon_inner: Arc::new(RwLock::new(TrayIconInner {
                is_close: false,
                is_visible: false,
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
                        println!("OnSettings!");
                        // lock.set_visible(true);
                        // self.set_visible(true);
                        tray.is_visible = true;
                        ctx.request_repaint();
                    }
                    TrayMessage::Exit => {
                        // self.set_close(true);
                        // lock.set_close(false);
                        println!("OnExit!");
                        tray.is_close = true;

                        ctx.request_repaint();
                    }
                    TrayMessage::OnIconDoubleClick => {
                        println!("OnIconDoubleClick!");
                        // self.set_visible(true);
                        // self.set_visible(true);
                        tray.is_visible = true;
                        ctx.request_repaint();
                    }
                    TrayMessage::OnIconClick => {
                        println!("OnIconClick!");
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
    fn on_close_event(&mut self) -> bool {
        self.set_visible(false);
        // self._tray_icon_inner
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
            println!("Start Tray  Event Monitor");
            self.tray_monitor(ctx.clone());
            self._tray_start = true;
        }

        if self.get_close() {
            _frame.close();
        }

        _frame.set_visible(self.get_visible());
        println!("Polling!");
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

fn custom_window_frame(
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    use egui::*;
    let text_color = ctx.style().visuals.text_color();

    // Height of the title bar
    let height = 28.0;

    CentralPanel::default()
        .frame(Frame::none())
        .show(ctx, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter();

            // Paint the frame:
            painter.rect(
                rect.shrink(1.0),
                10.0,
                ctx.style().visuals.window_fill(),
                Stroke::new(1.0, text_color),
            );

            // Paint the title:
            painter.text(
                rect.center_top() + vec2(0.0, height / 2.0),
                Align2::CENTER_CENTER,
                title,
                FontId::proportional(height * 0.8),
                text_color,
            );

            // Paint the line under the title:
            painter.line_segment(
                [
                    rect.left_top() + vec2(2.0, height),
                    rect.right_top() + vec2(-2.0, height),
                ],
                Stroke::new(1.0, text_color),
            );

            // Add the close button:
            // let close_response = ui.put(
            //     Rect::from_min_size(rect.left_top(), Vec2::splat(height)),
            //     Button::new(RichText::new("❌").size(height - 4.0)).frame(false),
            // );
            // if close_response.clicked() {
            //     frame.close();

            //     // frame.set_visible(false);
            // }

            // Interact with the title bar (drag to move window):
            let title_bar_rect = {
                let mut rect = rect;
                rect.max.y = rect.min.y + height;
                rect
            };
            let title_bar_response =
                ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            // Add the contents:
            let content_rect = {
                let mut rect = rect;
                rect.min.y = title_bar_rect.max.y;
                rect
            }
            .shrink(4.0);
            let mut content_ui = ui.child_ui(content_rect, *ui.layout());
            add_contents(&mut content_ui);
        });
}

pub fn ui_init() {
    // let native_options = eframe::NativeOptions::default();
    let native_options = eframe::NativeOptions {
        resizable: false,
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

    eframe::run_native(
        "Osic Window",
        native_options,
        Box::new(|cc| Box::new(App::new(cc, tray_icon, tray_rx))),
    );
}
