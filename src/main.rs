#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use adb_client::ADBUSBDevice;
use eframe::egui;
use egui::{Button, Layout, RichText, Sense, TextEdit};

fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_| Ok(Box::<App>::default())),
    )
}

struct App {
    search_query: String,
    entries: Vec<Entry>,
    device: Option<ADBUSBDevice>,
}

struct Entry {
    id: String,
    label: String,
    expand_triggered: bool,
    selected: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            device: None,
            search_query: "".to_owned(),

            entries: vec![
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
                Entry {
                    label: "Ding Dong game".to_string(),
                    id: "ding.dong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                },
            ],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let maybe_device = ADBUSBDevice::autodetect();
        if self.device.is_none() {
            if let Ok(device) = maybe_device {
                self.device.replace(device);
                println!("wow I found a device");
            } else {
                println!("looking ...");
            };
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);

            ui.take_available_width();
            ui.horizontal(|ui| {
                ui.take_available_width();
                ui.add_sized(
                    [ui.available_width(), 20.0],
                    TextEdit::singleline(&mut self.search_query).hint_text("Search"),
                )
            });

            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, entry) in self.entries.iter_mut().enumerate() {
                    render_entry(ui, entry, i);
                }
            });
        });
    }
}

fn render_entry(ui: &mut egui::Ui, entry: &mut Entry, index: usize) {
    let id = ui.make_persistent_id(format!("{}_state", index));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

    if entry.expand_triggered {
        state.toggle(ui);
        entry.expand_triggered = false;
    }
    state
        .show_header(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                let response = ui.add(
                    Button::selectable(entry.selected, RichText::new(&entry.label).size(12.0))
                        .right_text(RichText::new(&entry.id).monospace().size(10.0)),
                );
                let id = ui.make_persistent_id(format!("{}_interact", index));
                if ui
                    .interact(response.rect, id, Sense::click())
                    .double_clicked()
                {
                    entry.expand_triggered = true;
                    entry.selected ^= true;
                } else if ui.interact(response.rect, id, Sense::click()).clicked() {
                    entry.selected ^= true;
                }
            });
        })
        .body(|ui| ui.label("ping pong"));
}
