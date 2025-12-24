use crate::Action;
use crate::categories;
use crate::listview;

use egui::{
    Button, RichText, Spinner,
};

impl crate::App {
    pub fn action_bar(&mut self, ui: &mut egui::Ui) {
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

        let mut selected: Vec<&listview::Entry> = vec![];
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
                                .send(Action::Revert(
                                    entry.package.id.clone(),
                                    entry.strictly_disabled,
                                ))
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
    }
}
