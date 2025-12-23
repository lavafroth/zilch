#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    collections::{BTreeMap, BTreeSet},
    io::BufReader,
    sync::mpsc::{Receiver, Sender, channel},
    thread::{sleep, spawn},
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
const LABEL_EXTRACTOR: &[u8; 2124] = include_bytes!("./extractor.dex");

type FrontendPayload = PackageDiff;

struct App {
    search_query: String,
    uninstallable: bool,
    reinstallable: bool,
    entries: BTreeMap<String, Entry>,
    categories: u8,
    package_diff_rx: Receiver<FrontendPayload>,
    device_lost_rx: Receiver<()>,
    action_tx: Sender<Action>,
    // error_rx: Receiver<ShellRunError>,
    action_done_rx: Receiver<()>,
    disable_mode: bool,

    have_device: bool,
    busy: bool,
}

type PackageIdentifier = String;
type PackagePath = String;

#[derive(Clone)]
pub struct Package {
    id: PackageIdentifier,
    path: PackagePath,
    label: String,
}

struct Entry {
    package: Package,
    expand_triggered: bool,
    enabled: bool,
    selected: bool,
}

struct PackageDiff {
    added: Vec<Package>,
    removed: Vec<PackageIdentifier>,
}

#[derive(Debug)]
pub enum ShellRunError {
    Timeout,
    ParseError,
    Unrecoverable,
    UnsuccessfulOperation(PackageIdentifier),
    BackupNotPossible(PackageIdentifier),
    RevertFailed(PackageIdentifier),
}

mod categories {
    pub const RECOMMENDED: u8 = 0b10000;
    pub const ADVANCED: u8 = 0b01000;
    pub const EXPERT: u8 = 0b00100;
    pub const UNSAFE: u8 = 0b00010;
    pub const UNIDENTIFIED: u8 = 0b00001;

    pub const VALUES: [u8; 5] = [RECOMMENDED, ADVANCED, EXPERT, UNSAFE, UNIDENTIFIED];
    pub const NAMES: [&str; 5] = [
        "Recommended",
        "Advanced",
        "Expert",
        "Unsafe",
        "Unidentified",
    ];
}

pub enum Action {
    Uninstall(Package),
    Revert(PackageIdentifier),
    Disable(PackageIdentifier),
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Zilch",
        options,
        Box::new(|cc| {
            let (package_diff_tx, package_diff_rx) = channel();
            let (device_lost_tx, device_lost_rx) = channel();
            let (action_tx, action_rx) = channel();
            let (action_done_tx, action_done_rx) = channel();

            let ctx = cc.egui_ctx.clone();
            spawn(move || {
                worker_thread(
                    package_diff_tx,
                    device_lost_tx,
                    action_rx,
                    action_done_tx,
                    ctx,
                )
            });

            Ok(Box::new(App {
                busy: false,
                disable_mode: false,
                have_device: false,
                uninstallable: false,
                reinstallable: false,
                search_query: "".to_owned(),
                device_lost_rx,
                package_diff_rx,
                entries: Default::default(),
                action_tx,
                categories: categories::RECOMMENDED,
                action_done_rx,
            }))
        }),
    )
}

impl Action {
    fn apply_on_device(self, device: &mut ADBUSBDevice) -> Result<(), ShellRunError> {
        match self {
            Action::Uninstall(pkg) => {
                if pkg.path.is_empty() {
                    return Err(ShellRunError::BackupNotPossible(pkg.id));
                }

                let _copy_command_no_output = device.shell_command_text(&format!(
                    "cp {} /data/local/tmp/{}.apk",
                    pkg.path, pkg.id
                ))?;

                let output =
                    device.shell_command_text(&format!("pm uninstall --user 0 -k {}", pkg.id))?;

                if !output.contains("Success") {
                    return Err(ShellRunError::UnsuccessfulOperation(pkg.id));
                }
            }
            Action::Revert(id) => {
                let revert_command = format!("package install-existing {id}");
                let output = device.shell_command_text(&revert_command)?;

                if !output.contains("inaccessible or not found") {
                    return Ok(());
                }

                let revert_command = format!("pm install -r --user 0 /data/local/tmp/{id}.apk");
                let output = device.shell_command_text(&revert_command)?;
                if !output.contains("Success") {
                    return Err(ShellRunError::RevertFailed(id));
                }
            }
            Action::Disable(id) => {
                let disable_command = format!("pm disable --user 0 {id}");
                let output = device.shell_command_text(&disable_command)?;
                eprintln!("disable output {output:?}");
            }
        }
        Ok(())
    }
}

fn worker_thread(
    package_diff_tx: Sender<PackageDiff>,
    device_lost_tx: Sender<()>,
    action_rx: Receiver<Action>,
    action_done_tx: Sender<()>,
    ctx: egui::Context,
) {
    let mut maybe_device: Option<ADBUSBDevice> = None;
    let mut pkg_set: BTreeSet<PackageIdentifier> = Default::default();

    loop {
        while maybe_device.is_none() {
            maybe_device = ADBUSBDevice::autodetect().ok();
            if maybe_device.is_none() {
                sleep(WORKER_THREAD_POLL);
            }
        }

        if let Some(device) = maybe_device.as_mut() {
            let mut label_extractor_dex_stream = BufReader::new(&LABEL_EXTRACTOR[..]);
            let remote_path = "/data/local/tmp/extractor.dex";
            device
                .push(&mut label_extractor_dex_stream, &remote_path)
                .expect("failed to upload extractor to the device");
        }

        while let Some(device) = maybe_device.as_mut() {
            // do all the actions in bulk before the next render
            while let Ok(action) = action_rx.try_recv() {
                action.apply_on_device(device);
            }

            match fetch_packages(device, &pkg_set) {
                Ok((diff, new_pkg_set)) => {
                    if diff.added.is_empty() && diff.removed.is_empty() {
                        sleep(WORKER_THREAD_POLL);
                        continue;
                    }
                    pkg_set = new_pkg_set;
                    package_diff_tx.send(diff).expect("failed to send to ui");
                }
                Err(ShellRunError::Timeout) => {}
                Err(_log_this_later) => {
                    maybe_device = None;
                    pkg_set = BTreeSet::default();
                    device_lost_tx.send(()).expect("failed to send to ui");
                }
            }
            action_done_tx.send(()).expect("failed to send to ui");
            ctx.request_repaint();

            sleep(WORKER_THREAD_POLL);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        if let Ok(()) = self.action_done_rx.try_recv() {
            self.busy = false;
        }
        if let Ok(package_diff) = self.package_diff_rx.try_recv() {
            self.have_device = true;

            for package in package_diff.added {
                self.entries.insert(
                    package.id.clone(),
                    Entry {
                        package,
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
            self.entries.clear();

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

                for (id, entry) in self.entries.iter_mut() {
                    let query_lower = self.search_query.to_lowercase();
                    if id.to_lowercase().contains(&query_lower)
                        || entry.package.label.to_lowercase().contains(&query_lower)
                    {
                        render_entry(ui, entry);
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

            ui.style_mut().spacing.button_padding = [6.0, 6.0].into();
            ui.horizontal_wrapped(|ui| {
                for (category, bits) in categories::NAMES.into_iter().zip(categories::VALUES) {
                    let selected = self.categories & bits == bits;
                    if ui
                        .add(
                            Button::selectable(selected, RichText::new(category).size(12.0))
                                .corner_radius(10.0),
                        )
                        .clicked()
                    {
                        self.categories ^= bits;
                    };
                }
            });

            let button = if self.disable_mode {
                Button::new("disable")
            } else {
                Button::new("uninstall")
            };

            let mut selected: Vec<&Entry> = vec![];
            for entry in self.entries.values().filter(|entry| entry.selected) {
                selected.push(entry);
                if entry.enabled {
                    self.uninstallable = true;
                } else {
                    self.reinstallable = true;
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                let button_size = [80.0, 30.0];
                if self.busy {
                    ui.add_sized(button_size, Spinner::new());
                } else if self.uninstallable == self.reinstallable {
                    ui.add_enabled_ui(false, |ui| {
                        ui.add_sized(button_size, button);
                    });
                } else if self.uninstallable {
                    ui.add_enabled_ui(true, |ui| {
                        if !ui.add_sized(button_size, button).clicked() {
                            return;
                        }

                        if self.disable_mode {
                            for entry in selected.iter() {
                                self.action_tx
                                    .send(Action::Disable(entry.package.id.clone()))
                                    .expect("failed to send message to backend");
                            }
                        } else {
                            for entry in selected.iter() {
                                self.action_tx
                                    .send(Action::Uninstall(entry.package.clone()))
                                    .expect("failed to send message to backend");
                            }
                        }
                        self.busy = true;
                    });
                } else if self.reinstallable {
                    ui.add_enabled_ui(true, |ui| {
                        if ui.add_sized(button_size, Button::new("revert")).clicked() {
                            for entry in selected.iter() {
                                self.action_tx
                                    .send(Action::Revert(entry.package.id.clone()))
                                    .expect("failed to send message to backend");
                            }

                            self.busy = true;
                        }
                    });
                }

                ui.checkbox(&mut self.disable_mode, "disable mode")
                    .on_hover_text("prefer disabling apps to uninstalling");

                ui.separator();
                ui.label(format!("{} selected", selected.len()));
                ui.separator();
            });
            ui.add_space(2.0);
        });
    }
}

pub trait ShellCommandExt {
    fn shell_command_text(&mut self, command: &str) -> Result<String, ShellRunError>;
}

impl ShellCommandExt for ADBUSBDevice {
    fn shell_command_text(&mut self, command: &str) -> Result<String, ShellRunError> {
        let mut buf = Vec::with_capacity(4096);
        self.shell_command(&[command], &mut buf)
            .map_err(|e| match e {
                adb_client::RustADBError::UsbError(rusb::Error::Timeout) => ShellRunError::Timeout,
                _ => ShellRunError::Unrecoverable,
            })?;
        String::from_utf8(buf).map_err(|_| ShellRunError::ParseError)
    }
}

fn fetch_packages(
    device: &mut ADBUSBDevice,
    pkg_set: &BTreeSet<String>,
) -> Result<(PackageDiff, BTreeSet<String>), ShellRunError> {
    let raw_pkg_text = device.shell_command_text("pm list packages -f")?;

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
        let raw_pkg_text = device
            .shell_command_text("CLASSPATH=/data/local/tmp/extractor.dex app_process / Main")?;

        for line in raw_pkg_text.lines() {
            let mut splitn = line.splitn(3, ' ');
            // discard
            splitn.next();
            let id = splitn.next().expect("split n=3 does not have 3 elements");
            let label = splitn.next().expect("split n=3 does not have 3 elements");

            if let Some(package_mut) = new_packages.get_mut(id) {
                package_mut.label = label.to_string();
            }
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
    let mut label = RichText::new(format!("{}\n", entry.package.label)).size(12.0);
    let mut package_id = RichText::new(&entry.package.id).monospace().size(10.0);
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
    let id = ui.make_persistent_id(format!("{}_state", entry.package.id));
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
            let id = ui.make_persistent_id(format!("{}_interact", entry.package.id));
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
