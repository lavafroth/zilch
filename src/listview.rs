use crate::Metadata;
use crate::Package;
use crate::categories;
use egui::{Align, Button, Color32, Layout, RichText, Sense, Stroke, Style, text::LayoutJob};

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum State {
    Enabled = 0b001,
    Uninstalled = 0b010,
    Disabled = 0b110,
}

pub struct Entry {
    pub package: Package,
    pub expand_triggered: bool,
    pub state: State,
    pub selected: bool,
    pub metadata: Option<&'static Metadata>,
}

impl Entry {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        let id = ui.make_persistent_id(format!("{}_state", self.package.id));
        let mut state =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);

        if self.expand_triggered {
            state.toggle(ui);
            self.expand_triggered = false;
        }

        let header = ui.horizontal(|ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(20.0, 10.0);
            ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                // supply faint versions of the colors for disabled apps
                let faint_bg = ui.style().visuals.faint_bg_color;
                let selection_bg = ui.style().visuals.selection.bg_fill;
                let fg = ui.style().visuals.selection.stroke.color;
                let faint_selection_bg = selection_bg.lerp_to_gamma(faint_bg, 0.6);
                let faint_fg = fg.lerp_to_gamma(faint_bg, 0.6);

                let response = ui.add(self.button(
                    faint_selection_bg,
                    selection_bg,
                    faint_fg,
                    categories::value_to_name(self.metadata.map(|m| m.removal).unwrap_or_default()),
                ));
                let id = ui.make_persistent_id(format!("{}_interact", self.package.id));
                if ui
                    .interact(response.rect, id, Sense::click())
                    .double_clicked()
                {
                    self.expand_triggered = true;
                    self.selected ^= true;
                } else if ui.interact(response.rect, id, Sense::click()).clicked() {
                    self.selected ^= true;
                }
            });
        });

        state.show_body_indented(&header.response, ui, |ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new(
                    self.metadata
                        .map(|m| m.description)
                        .unwrap_or("Description unavailable."),
                )
                .size(12.0),
            );
            ui.add_space(4.0);
        });
    }

    fn button<'a>(
        &self,
        faint_bg: Color32,
        selection_bg: Color32,
        faint_fg: Color32,
        right_text: &'a str,
    ) -> Button<'a> {
        let mut job = LayoutJob::default();
        let mut label = RichText::new(format!("{}\n", self.package.label)).size(12.0);
        let mut package_id = RichText::new(&self.package.id).monospace().size(10.0);
        let mut right_text = RichText::new(right_text);

        if self.state != State::Enabled {
            label = label.strikethrough().color(faint_fg);
            package_id = package_id.strikethrough().color(faint_fg);
            right_text = right_text.color(faint_fg);
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
        let button = Button::selectable(self.selected, job);
        if self.state != State::Enabled {
            button.stroke(Stroke::new(1.0, selection_bg)).fill(faint_bg)
        } else {
            button
        }
        .right_text(right_text)
    }
}
