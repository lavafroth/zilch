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

        if ui.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl)
            && let Some(path) = rfd::FileDialog::new()
                .set_file_name("zilch.ini")
                .save_file()
        {
            let mut enabled = vec![];
            let mut uninstalled = vec![];
            let mut disabled = vec![];
            for (id, entry) in self.entries.iter() {
                if entry.enabled {
                    enabled.push(id.clone());
                    continue;
                }

                if entry.strictly_disabled {
                    disabled.push(id.clone());
                    continue;
                }

                uninstalled.push(id.clone());
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
    }
}
