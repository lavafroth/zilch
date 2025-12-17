#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use adb_client::ADBUSBDevice;
use eframe::egui;
use egui::{
    Align, Button, Color32, Label, Layout, RichText, Sense, Spinner, TextEdit, TopBottomPanel,
};
use egui_alignments::{center_horizontal, column};

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
    uninstallable: bool,
    reinstallable: bool,
}

struct Entry {
    id: String,
    label: String,
    expand_triggered: bool,
    enabled: bool,
    selected: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            device: None,
            uninstallable: false,
            reinstallable: false,
            search_query: "".to_owned(),

            entries: vec![
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: true,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: true,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: true,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: false,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: true,
                },
                Entry {
                    label: "Ping Pong game".to_string(),
                    id: "ping.pong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: true,
                },
                Entry {
                    label: "Ding Dong game".to_string(),
                    id: "ding.dong.bell".to_string(),
                    expand_triggered: false,
                    selected: false,
                    enabled: true,
                },
            ],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        let debug = true;
        // let debug = false;
        if !debug && self.device.is_none() {
            egui::CentralPanel::default().show(ctx, |ui| {
                center_horizontal(ui, |ui| {
                    column(ui, Align::Center, |ui| {
                    ui.add(Spinner::new());
                    ui.add(Label::new("Waiting for a device.\nPlease connect your Android device via USB ensuring\nthat USB debugging is enabled in developer settings."));
                    });
                });
            });

            self.device = ADBUSBDevice::autodetect().ok();
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
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
                self.uninstallable = false;
                self.reinstallable = false;
                for (i, entry) in self.entries.iter_mut().enumerate() {
                    render_entry(ui, entry, i);
                    if !entry.selected {
                        continue;
                    }
                    if entry.enabled {
                        self.uninstallable = true;
                    } else {
                        self.reinstallable = true;
                    }
                }
            });
        });
        TopBottomPanel::bottom("action_bar").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if self.uninstallable && self.reinstallable {
                    ui.add_enabled(false, Button::new("uninstall"));
                    ui.add_enabled(false, Button::new("disable"));
                } else if self.uninstallable {
                    ui.add_enabled(true, Button::new("uninstall"));
                    ui.add_enabled(true, Button::new("disable"));
                } else if self.reinstallable {
                    ui.add_enabled(true, Button::new("revert"));
                    ui.add_enabled(false, Button::new("disable"));
                } else {
                    ui.add_enabled(false, Button::new("uninstall"));
                    ui.add_enabled(false, Button::new("disable"));
                }
            });
            ui.add_space(2.0);
        });
    }
}

fn create_button(entry: &'_ Entry) -> Button<'_> {
    let label = RichText::new(&entry.label).size(12.0);
    let package_id = RichText::new(&entry.id).monospace().size(10.0);
    let disabled_text_color = Color32::from_rgb(100, 100, 100);
    if entry.enabled {
        Button::selectable(entry.selected, label).right_text(package_id)
    } else {
        Button::selectable(
            entry.selected,
            label.strikethrough().color(disabled_text_color),
        )
        .fill(Color32::from_rgb(60, 60, 60))
        .right_text(package_id.strikethrough().color(disabled_text_color))
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
                let response = ui.add(create_button(&entry));
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
        .body(|ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("a very long description that only the real arch wiki level nerds will bother to read lol.").size(12.0));
            ui.add_space(4.0);
        });
}
