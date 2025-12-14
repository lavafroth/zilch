#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;
use egui::{Button, Layout, Sense, TextEdit};

fn main() -> eframe::Result {
    // env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    name: String,
    entries: Vec<Entry>,
}

struct Entry {
    id: String,
    expanded: bool,
    selected: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "someapp".to_owned(),

            entries: vec![
                Entry {
                    id: "ping.pong.bell".to_string(),
                    expanded: false,
                    selected: false,
                },
                Entry {
                    id: "ding.dong.bell".to_string(),
                    expanded: false,
                    selected: false,
                },
            ],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ctx.set_pixels_per_point(1.5);
            // self.ctrl = ui.input(|i| i.modifiers.ctrl);

            ui.take_available_width();
            ui.horizontal(|ui| {
                ui.take_available_width();
                let search_label = ui.label("Search: ");
                ui.add_sized(
                    [ui.available_width(), 20.0],
                    TextEdit::singleline(&mut self.name),
                )
                .labelled_by(search_label.id);
            });

            ui.separator();

            for entry in self.entries.iter_mut() {
                render_entry(ui, entry);
            }
        });
    }
}

fn render_entry(ui: &mut egui::Ui, entry: &mut Entry) {
    let id = ui.make_persistent_id(&format!("{}_state", entry.id));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

    if entry.expanded {
        state.toggle(ui);
        entry.expanded = false;
    }
    state
        .show_header(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                let response = ui.selectable_label(entry.selected, &entry.id);
                let id = ui.make_persistent_id(&format!("{}_interact", entry.id));
                if ui
                    .interact(response.rect, id, Sense::click())
                    .double_clicked()
                {
                    entry.expanded = true;
                } else if ui.interact(response.rect, id, Sense::click()).clicked() {
                    entry.selected ^= true;
                }
            });
        })
        .body(|ui| ui.label("ping pong"));
}
