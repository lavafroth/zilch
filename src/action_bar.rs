use crate::Action;
use crate::categories;
use crate::listview;
use crate::listview::State;

use egui::Vec2;
use egui::{Button, RichText, Spinner};

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
        let mut selected_app_state = 0b111;
        for entry in self.entries.values().filter(|entry| entry.selected) {
            selected.push(entry);
            selected_app_state &= entry.state as u8;
        }

        // eprintln!("{0:08b}", self.selected_app_state);

        ui.separator();

        ui.horizontal(|ui| {
            let button_size = [80.0, 30.0].into();
            if self.busy {
                ui.add_sized(button_size, Spinner::new());
            } else if selected_app_state == State::Enabled as u8 {
                if add_enabled_button(true, ui, button_size, button) {
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
                }
            } else if selected_app_state == State::Uninstalled as u8 {
                if add_enabled_button(true, ui, button_size, Button::new("revert")) {
                    for entry in selected.iter() {
                        self.action_tx
                            .send(Action::Revert(entry.package.id.clone(), entry.state))
                            .expect("failed to send message to backend");
                    }

                    self.busy = true;
                }
            } else {
                // the selection is a mix of enabled and disabled apps:
                // gray out the button
                add_enabled_button(false, ui, button_size, button);
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

fn add_enabled_button(enabled: bool, ui: &mut egui::Ui, size: Vec2, button: Button) -> bool {
    ui.add_enabled_ui(enabled, |ui| ui.add_sized(size, button))
        .inner
        .clicked()
}
