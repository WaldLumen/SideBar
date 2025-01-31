use crate::ui::color_parser::parse_color_from_ini;
use eframe::glow::Context;
use egui::{Frame, Pos2, TextEdit, Vec2, Window};
use std::process::Command;

#[derive(Default)]
pub(crate) struct Settings {
    pub theme: String,
    pub owm_api_key: String,
    pub city: String,
}

impl Settings {
    pub fn create_settings_window(&mut self, ctx: &egui::Context) {
        Window::new("Edit Task")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(300.0, 500.0))
            .show(ctx, |ui| {
                ui.label("Reminder Text:");
                ui.add(TextEdit::multiline(&mut self.city).min_size(Vec2::new(300.0, 100.0)));

                ui.label("Reminder Time:");
                ui.add(TextEdit::multiline(&mut self.theme).min_size(Vec2::new(300.0, 50.0)));

                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    if ui.button("Save").clicked() {}

                    if ui.button("Close").clicked() {}
                });
            });
    }
}
