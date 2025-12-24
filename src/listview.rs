use crate::Metadata;
use crate::Package;
use crate::categories;
use egui::{
    Align, Button, Color32, Layout, RichText, Sense, Stroke, Style, text::LayoutJob,
};

pub(crate) struct Entry {
    pub package: Package,
    pub expand_triggered: bool,
    pub enabled: bool,
    pub selected: bool,
    pub metadata: Option<&'static Metadata>,
    pub strictly_disabled: bool,
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
                let faint_bg = ui.style().visuals.faint_bg_color;
                let selection_bg = ui.style().visuals.selection.bg_fill;
                let faint_selection_bg = selection_bg.lerp_to_gamma(faint_bg, 0.6);

                let response = ui.add(
                    create_button(self, faint_selection_bg, selection_bg).right_text(
                        categories::value_to_name(
                            self.metadata.map(|m| m.removal).unwrap_or_default(),
                        ),
                    ),
                );
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
}

fn create_button(entry: &'_ Entry, faint_bg: Color32, selection_bg: Color32) -> Button<'_> {
    let mut job = LayoutJob::default();
    let mut label = RichText::new(format!("{}\n", entry.package.label)).size(12.0);
    let mut package_id = RichText::new(&entry.package.id).monospace().size(10.0);

    if !entry.enabled {
        label = label.strikethrough();
        package_id = package_id.strikethrough();
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
        button.stroke(Stroke::new(1.0, selection_bg)).fill(faint_bg)
    } else {
        button
    }
}
