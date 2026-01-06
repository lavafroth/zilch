#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in releaspackage

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::BufReader,
    sync::mpsc::{Receiver, Sender, channel},
    thread::{sleep, spawn},
    time::Duration,
};

use adb_client::{ADBDeviceExt, ADBUSBDevice};
use eframe::egui;
use egui::{Align, CentralPanel, Label, Spinner, TextEdit, TopBottomPanel};
use egui_alignments::{center_horizontal, column};

use crate::{action::Action, adb_shell_text::ShellCommandText};
mod action;
mod action_bar;
mod adb_shell_text;
mod categories;
mod listview;
mod metadata;
mod shortcuts;

const WORKER_THREAD_POLL: Duration = Duration::from_secs(5);
const LABEL_EXTRACTOR: &[u8; 2124] = include_bytes!("./extractor.dex");

type FrontendPayload = PackageDiff;

struct App {
    search_query: String,
    entries: BTreeMap<String, listview::Entry>,
    categories: u8,

    package_diff_rx: Receiver<FrontendPayload>,
    device_lost_rx: Receiver<()>,
    action_tx: Sender<Action>,
    action_error_rx: Receiver<ShellRunError>,
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

struct PackageDiff {
    added: Vec<Package>,
    removed: Vec<PackageIdentifier>,
    disabled: Vec<String>,
    re_enabled: Vec<String>,
}

impl PackageDiff {
    fn same_as_before(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.disabled.is_empty()
            && self.re_enabled.is_empty()
    }
}

pub enum ShellRunError {
    Timeout,
    ParseError,
    Unrecoverable,
    UninstallFailed(PackageIdentifier),
    BackupNotPossible(PackageIdentifier),
    RevertFailed(PackageIdentifier),
    DisableFailed(PackageIdentifier),
}

impl Display for ShellRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellRunError::Timeout => f.write_str("timed out in running shell command on device"),
            ShellRunError::ParseError => {
                f.write_str("failed to parse the output of shell command from the device")
            }
            ShellRunError::Unrecoverable => f.write_str("unrecoverable error"),
            ShellRunError::UninstallFailed(id) => write!(f, "failed to uninstall package {id}"),
            ShellRunError::BackupNotPossible(id) => {
                write!(f, "failed to backup package {id} before uninstall")
            }
            ShellRunError::RevertFailed(id) => write!(f, "failed to revert package {id}"),
            ShellRunError::DisableFailed(id) => write!(f, "failed to disable package {id}"),
        }
    }
}

#[derive(Debug)]
pub struct Metadata {
    description: &'static str,
    removal: u8,
}

fn main() -> eframe::Result {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
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
            let (action_result_tx, action_result_rx) = channel();

            let ctx = cc.egui_ctx.clone();
            spawn(move || {
                worker_thread(
                    package_diff_tx,
                    device_lost_tx,
                    action_rx,
                    action_done_tx,
                    action_result_tx,
                    ctx,
                )
            });

            Ok(Box::new(App {
                busy: false,
                disable_mode: false,
                have_device: false,
                search_query: "".to_owned(),
                device_lost_rx,
                package_diff_rx,
                entries: Default::default(),
                action_tx,
                categories: categories::RECOMMENDED,
                action_done_rx,
                action_error_rx: action_result_rx,
            }))
        }),
    )
}

fn worker_thread(
    package_diff_tx: Sender<PackageDiff>,
    device_lost_tx: Sender<()>,
    action_rx: Receiver<Action>,
    action_done_tx: Sender<()>,
    action_error_tx: Sender<ShellRunError>,
    ctx: egui::Context,
) {
    let mut maybe_device: Option<ADBUSBDevice> = None;
    let mut pkg_set: BTreeSet<PackageIdentifier> = Default::default();
    let mut disabled_set: BTreeSet<PackageIdentifier> = Default::default();
    let mut device_version = 0;

    loop {
        while maybe_device.is_none() {
            maybe_device = ADBUSBDevice::autodetect().ok();
            if maybe_device.is_none() {
                sleep(WORKER_THREAD_POLL);
            }
        }

        if let Some(device) = maybe_device.as_mut() {
            let Ok(sdk_version_str) = device.shell_command_text("getprop ro.build.version.sdk")
            else {
                log::error!("failed to get android version from the device");
                maybe_device = None;
                continue;
            };

            let Ok(sdk_version) = sdk_version_str.trim().parse::<u16>() else {
                log::error!("failed to parse device android sdk version");
                maybe_device = None;
                continue;
            };

            device_version = sdk_version;

            let mut label_extractor_dex_stream = BufReader::new(&LABEL_EXTRACTOR[..]);
            let remote_path = "/data/local/tmp/extractor.dex";
            device
                .push(&mut label_extractor_dex_stream, &remote_path)
                .expect("failed to upload extractor to the device");
        }

        while let Some(device) = maybe_device.as_mut() {
            // do all the actions in bulk before the next render
            while let Ok(action) = action_rx.try_recv() {
                if let Err(action_error) = action.apply_on_device(device) {
                    action_error_tx
                        .send(action_error)
                        .expect("failed to send to ui");
                }
            }

            action_done_tx.send(()).expect("failed to send to ui");
            match fetch_packages(device, &pkg_set, &disabled_set) {
                Ok((diff, new_pkg_set, new_disabled_set)) => {
                    if diff.same_as_before() {
                        sleep(WORKER_THREAD_POLL);
                        continue;
                    }
                    pkg_set = new_pkg_set;
                    disabled_set = new_disabled_set;
                    package_diff_tx.send(diff).expect("failed to send to ui");
                }
                Err(ShellRunError::Timeout) => {}
                Err(_log_this_later) => {
                    maybe_device = None;
                    pkg_set = BTreeSet::default();
                    device_lost_tx.send(()).expect("failed to send to ui");
                }
            }
            ctx.request_repaint();

            sleep(WORKER_THREAD_POLL);
        }
    }
}

impl App {
    fn reconcile(&mut self, package_diff: PackageDiff) {
        for package in package_diff.added {
            let maybe_meta = metadata::STORE.get(&package.id);
            self.entries.insert(
                package.id.clone(),
                listview::Entry {
                    package,
                    metadata: maybe_meta,
                    expand_triggered: false,
                    state: listview::State::Enabled,
                    selected: false,
                },
            );
        }

        for package_id in package_diff.removed {
            if let Some(entry) = self.entries.get_mut(&package_id) {
                entry.state = listview::State::Uninstalled;
            };
        }

        for package_id in package_diff.disabled {
            if let Some(entry) = self.entries.get_mut(&package_id) {
                entry.state = listview::State::Disabled;
            };
        }

        for package_id in package_diff.re_enabled {
            if let Some(entry) = self.entries.get_mut(&package_id) {
                entry.state = listview::State::Enabled;
            };
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        if let Ok(()) = self.action_done_rx.try_recv() {
            self.busy = false;
        }

        if let Ok(action_error) = self.action_error_rx.try_recv() {
            log::error!("{}", action_error.to_string());
        }

        if let Ok(package_diff) = self.package_diff_rx.try_recv() {
            self.have_device = true;
            self.reconcile(package_diff);
        }

        if let Ok(()) = self.device_lost_rx.try_recv() {
            self.have_device = false;
            self.entries.clear();

            log::warn!("device lost");
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

        TopBottomPanel::bottom("action_bar").show(ctx, |ui| self.action_bar(ui));

        CentralPanel::default().show(ctx, |ui| {
            ui.take_available_width();
            let search = ui.horizontal(|ui| {
                ui.take_available_width();
                ui.add_sized(
                    [ui.available_width(), 20.0],
                    TextEdit::singleline(&mut self.search_query).hint_text("Search"),
                )
            });

            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (id, entry) in self.entries.iter_mut() {
                    let query_lower = self.search_query.to_lowercase();
                    let entry_removal = entry
                        .metadata
                        .map(|m| m.removal)
                        .unwrap_or(categories::UNIDENTIFIED);
                    if (id.to_lowercase().contains(&query_lower)
                        || entry.package.label.to_lowercase().contains(&query_lower))
                        && (entry_removal & self.categories == entry_removal)
                    {
                        entry.render(ui);
                    }
                }
            });
            self.handle_shortcuts(ui, search.response);
        });
    }
}

fn fetch_packages(
    device: &mut ADBUSBDevice,
    pkg_set: &BTreeSet<String>,
    disabled_set: &BTreeSet<String>,
) -> Result<(PackageDiff, BTreeSet<String>, BTreeSet<String>), ShellRunError> {
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

    // disabled
    let raw_pkg_text = device.shell_command_text("pm list packages -d")?;
    let mut current_disabled_set = BTreeSet::new();
    for line in raw_pkg_text.lines() {
        let id = line.strip_prefix("package:").unwrap_or(line);
        current_disabled_set.insert(id.to_string());
    }

    let re_enabled = disabled_set
        .difference(&current_disabled_set)
        .map(|v| v.to_string())
        .collect();

    let disabled = current_disabled_set
        .difference(disabled_set)
        .map(|v| v.to_string())
        .collect();

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
            disabled,
            re_enabled,
        },
        current_set,
        current_disabled_set,
    ))
}
