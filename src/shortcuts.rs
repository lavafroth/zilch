use crate::action::Action;

impl crate::App {
    pub fn handle_shortcuts(&mut self, ui: &mut egui::Ui, search_modal: egui::Response) {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            for (_id, entry) in self.entries.iter_mut() {
                entry.selected = false;
            }
        }
        if ui.input(|i| {
            i.key_pressed(egui::Key::S)
                || i.key_pressed(egui::Key::Slash)
                || (i.modifiers.ctrl && i.key_pressed(egui::Key::F))
        }) {
            search_modal.request_focus();
        }

        if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            self.save_config();
        }
    }

    pub fn save_config(&self) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("zilch.ini")
            .save_file()
        else {
            return;
        };
        let mut enabled = vec![];
        let mut uninstalled = vec![];
        let mut disabled = vec![];
        for (id, entry) in self.entries.iter() {
            match entry.state {
                crate::listview::State::Enabled => enabled.push(id.clone()),
                crate::listview::State::Disabled => disabled.push(id.clone()),
                crate::listview::State::Uninstalled => uninstalled.push(id.clone()),
            }
        }

        let contents = format!(
            "disabled={}\nenabled={}\nuninstalled={}",
            disabled.join(","),
            enabled.join(","),
            uninstalled.join(",")
        );
        if let Err(e) = std::fs::write(&path, contents) {
            eprintln!("failed to write device state to {}: {e}", path.display());
        };
    }

    pub fn import_config(&mut self) {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            return;
        };
        let contents = match std::fs::read_to_string(&path) {
            Err(e) => {
                eprintln!(
                    "failed to read device state config from path {}: {e}",
                    path.display()
                );
                return;
            }
            Ok(s) => s,
        };

        self.busy = true;
        for line in contents.lines() {
            let Some((state, csv)) = line.split_once('=') else {
                continue;
            };

            if state == "disabled" {
                for entry in csv.split(',') {
                    self.action_tx
                        .send(Action::Disable(entry.to_string()))
                        .expect(&format!(
                            "failed to send message to backend for disabling {entry} during import"
                        ));
                }
            }

            if state == "uninstalled" {
                for pkgid in csv.split(',') {
                    let Some(entry) = self.entries.get(pkgid) else {
                        continue;
                    };
                    self.action_tx
                        .send(Action::Uninstall(entry.package.clone()))
                        .expect(&format!(
                            "failed to send message to backend for uninstalling {} during import",
                            pkgid
                        ));
                }
            }
        }

        self.busy = false;
    }
}
