#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufReader, Write},
    sync::mpsc::{Receiver, channel},
    thread::sleep,
    time::Duration,
};

use adb_client::{ADBDeviceExt, ADBUSBDevice};
use eframe::egui;
use egui::{
    Align, Button, Color32, Label, Layout, RichText, Sense, Spinner, Style, TextEdit,
    TopBottomPanel, text::LayoutJob,
};
use egui_alignments::{center_horizontal, column};

const WORKER_THREAD_POLL: Duration = Duration::from_secs(5);

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let (package_diff_tx, package_diff_rx) = channel();
            let (device_lost_tx, device_lost_rx) = channel();

            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let mut device: Option<ADBUSBDevice> = None;
                let mut pkg_set: BTreeSet<String> = Default::default();

                loop {
                    while device.is_none() {
                        device = ADBUSBDevice::autodetect().ok();

                        sleep(WORKER_THREAD_POLL);
                    }

                    if let Some(device_mut) = device.as_mut() {
                        let mut label_extractor_dex_stream = BufReader::new(&LABEL_EXTRACTOR[..]);
                        let remote_path = "/data/local/tmp/extractor.dex";
                        device_mut
                            .push(&mut label_extractor_dex_stream, &remote_path)
                            .expect("failed to upload extractor to the device");
                    }

                    while let Some(device_mut) = device.as_mut() {
                        match fetch_packages(device_mut, &pkg_set) {
                            Ok((diff, new_pkg_set)) => {
                                if diff.added.is_empty() && diff.removed.is_empty() {
                                    sleep(WORKER_THREAD_POLL);
                                    continue;
                                }
                                pkg_set = new_pkg_set;
                                package_diff_tx.send(diff).expect("failed to send to ui");
                            }
                            Err(FetchPackageError::Timeout) => {}
                            Err(_log_this_later) => {
                                device = None;
                                pkg_set = BTreeSet::default();
                                device_lost_tx.send(()).expect("failed to send to ui");
                            }
                        }
                        ctx.request_repaint();

                        sleep(WORKER_THREAD_POLL);
                    }
                }
            });

            Ok(Box::new(App {
                n_selected: 0,
                disable_mode: false,
                have_device: false,
                uninstallable: false,
                reinstallable: false,
                search_query: "".to_owned(),
                device_lost_rx,
                package_diff_rx,
                entries: Default::default(),
            }))
        }),
    )
}

type FrontendPayload = PackageDiff;

struct App {
    search_query: String,
    uninstallable: bool,
    reinstallable: bool,
    n_selected: usize,
    entries: BTreeMap<String, Entry>,
    package_diff_rx: Receiver<FrontendPayload>,
    device_lost_rx: Receiver<()>,
    disable_mode: bool,

    have_device: bool,
}

#[derive(Clone)]
struct Package {
    id: String,
    path: String,
    label: String,
}

struct Entry {
    id: String,
    label: String,
    expand_triggered: bool,
    enabled: bool,
    selected: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);
        if let Ok(package_diff) = self.package_diff_rx.try_recv() {
            self.have_device = true;

            for package in package_diff.added {
                self.entries.insert(
                    package.id.clone(),
                    Entry {
                        id: package.id,
                        label: package.label,
                        expand_triggered: false,
                        enabled: true,
                        selected: false,
                    },
                );
            }

            for package_id in package_diff.removed {
                if let Some(entry) = self.entries.get_mut(&package_id) {
                    entry.enabled = false;
                };
            }
        }

        if let Ok(()) = self.device_lost_rx.try_recv() {
            self.have_device = false;
            println!("device lost");
        }

        if !self.have_device {
            egui::CentralPanel::default().show(ctx, |ui| {
                center_horizontal(ui, |ui| {
                    column(ui, Align::Center, |ui| {
                    ui.add(Spinner::new());
                    ui.add(Label::new("Waiting for a device.\nPlease connect your Android device via USB ensuring\nthat USB debugging is enabled in developer settings."));
                    });
                });
            });
            return;
        };

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
                self.n_selected = 0;

                for (_id, entry) in self.entries.iter_mut() {
                    render_entry(ui, entry);
                    if !entry.selected {
                        continue;
                    }

                    self.n_selected += 1;
                    if entry.enabled {
                        self.uninstallable = true;
                    } else {
                        self.reinstallable = true;
                    }
                }
            });

            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                for (_id, entry) in self.entries.iter_mut() {
                    entry.selected = false;
                }
            }
        });
        TopBottomPanel::bottom("action_bar").show(ctx, |ui| {
            ui.add_space(6.0);

            let button = if self.disable_mode {
                Button::new("disable")
            } else {
                Button::new("uninstall")
            };

            ui.horizontal(|ui| {
                if self.uninstallable == self.reinstallable {
                    ui.add_enabled(false, button);
                } else if self.uninstallable {
                    ui.add_enabled(true, button);
                } else if self.reinstallable {
                    ui.add_enabled(true, Button::new("revert"));
                }

                ui.checkbox(&mut self.disable_mode, "disable mode")
                    .on_hover_text("prefer disabling apps to uninstalling");

                ui.separator();
                ui.label(format!("{} selected", self.n_selected));
                ui.separator();
            });
            ui.add_space(2.0);
        });
    }
}

const LABEL_EXTRACTOR: &[u8; 2124] = include_bytes!("./extractor.dex");

struct PackageDiff {
    added: Vec<Package>,
    removed: Vec<String>,
}

pub enum FetchPackageError {
    Timeout,
    ParseError,
    Unrecoverable,
}

pub trait ShellCommandExt {
    fn shell_command_ext(
        &mut self,
        command: &str,
        buf: &mut dyn Write,
    ) -> Result<(), FetchPackageError>;
}

impl ShellCommandExt for ADBUSBDevice {
    fn shell_command_ext(
        &mut self,
        command: &str,
        buf: &mut dyn Write,
    ) -> Result<(), FetchPackageError> {
        self.shell_command(&[command], buf).map_err(|e| match e {
            adb_client::RustADBError::UsbError(rusb::Error::Timeout) => FetchPackageError::Timeout,
            _ => FetchPackageError::Unrecoverable,
        })
    }
}

fn fetch_packages(
    device: &mut ADBUSBDevice,
    pkg_set: &BTreeSet<String>,
) -> Result<(PackageDiff, BTreeSet<String>), FetchPackageError> {
    let mut buffer = Vec::with_capacity(1024);
    device.shell_command_ext("pm list packages -f", &mut buffer)?;
    let raw_pkg_text = std::str::from_utf8(&buffer).map_err(|_| FetchPackageError::ParseError)?;

    let mut current_set = BTreeSet::new();

    let mut new_packages = BTreeMap::new();
    for line in raw_pkg_text.lines() {
        let stripped = line.strip_prefix("package:").unwrap_or(line);
        let (path, id) = stripped.rsplit_once("=").unwrap_or((line, ""));
        current_set.insert(id.to_string());

        if !pkg_set.contains(id) {
            let package = Package {
                path: path.to_string(),
                id: id.to_string(),
                label: String::default(),
            };

            new_packages.insert(id, package);
        }
    }

    let removed = pkg_set.difference(&current_set).cloned().collect();

    let need_to_fetch_labels = !new_packages.is_empty();
    if need_to_fetch_labels {
        let mut buffer = Vec::with_capacity(1024);
        device.shell_command_ext(
            "CLASSPATH=/data/local/tmp/extractor.dex app_process / Main",
            &mut buffer,
        )?;
        let raw_pkg_text =
            std::str::from_utf8(&buffer).map_err(|_| FetchPackageError::ParseError)?;

        for line in raw_pkg_text.lines() {
            let mut splitn = line.splitn(3, ' ');
            // discard
            splitn.next();
            let id = splitn.next().expect("split n=3 does not have 3 elements");
            let label = splitn.next().expect("split n=3 does not have 3 elements");

            if let Some(package_mut) = new_packages.get_mut(id) {
                package_mut.label = label.to_string();
            }
            // eprintln!("{line}");
        }
    }

    Ok((
        PackageDiff {
            added: new_packages.into_values().collect(),
            removed,
        },
        current_set,
    ))
}

fn create_button(entry: &'_ Entry) -> Button<'_> {
    let mut job = LayoutJob::default();
    let mut label = RichText::new(format!("{}\n", entry.label)).size(12.0);
    let mut package_id = RichText::new(&entry.id).monospace().size(10.0);
    let disabled_text_color = Color32::from_rgb(100, 100, 100);

    if !entry.enabled {
        label = label.strikethrough().color(disabled_text_color);
        package_id = package_id.strikethrough().color(disabled_text_color);
    }

    label.append_to(
        &mut job,
        &Style::default(),
        egui::FontSelection::Default,
        Align::Min,
    );
    package_id.append_to(
        &mut job,
        &Style::default(),
        egui::FontSelection::Default,
        Align::Min,
    );
    let button = Button::selectable(entry.selected, job);
    if !entry.enabled {
        button.fill(Color32::from_rgb(60, 60, 60))
    } else {
        button
    }
}

fn render_entry(ui: &mut egui::Ui, entry: &mut Entry) {
    let id = ui.make_persistent_id(format!("{}_state", entry.id));
    let mut state =
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

    if entry.expand_triggered {
        state.toggle(ui);
        entry.expand_triggered = false;
    }

    let header_res = ui.horizontal(|ui| {
        ui.style_mut().spacing.button_padding = egui::vec2(20.0, 10.0);
        ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
            let response = ui.add(create_button(entry));
            let id = ui.make_persistent_id(format!("{}_interact", entry.id));
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
    });

    state.show_body_indented(&header_res.response, ui, |ui| {
            ui.add_space(4.0);
            ui.label(RichText::new("a very long description that only the real arch wiki level nerds will bother to read lol.").size(12.0));
            ui.add_space(4.0);
        });
}
