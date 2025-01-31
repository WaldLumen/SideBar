use crate::ui::color_parser::parse_color_from_ini;
use configparser::ini::{Ini, WriteOptions};
use egui::{Frame, Pos2, TextEdit, Ui, Vec2, Window};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;

#[derive(Default)]
pub(crate) struct Settings {
    pub theme: String,
    pub themes: Vec<String>,
    pub owm_api_key: String,
    pub city: String,
    pub popup_open: bool,
    pub first_open: bool,
}

impl Settings {
    pub fn create_settings_window(&mut self, ctx: &egui::Context) -> Result<(), Box<dyn Error>> {
        let mut settings = Ini::new();
        let mut themes = Ini::new();

        let home = env::var("HOME").expect("Could not determine the home directory");
        let theme_path = format!("{}/.config/sidebar/themes.ini", home);
        let settings_path = format!("{}/.config/sidebar/settings.ini", home);

        themes.load(theme_path.clone())?;
        settings.load(settings_path.clone())?;
        if !self.first_open {
            self.themes = themes.sections();
            self.first_open = true;
        }
        Window::new("Edit Theme")
            .title_bar(false)
            .default_pos(egui::pos2(75.0, 300.0))
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(300.0, 500.0))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label("Select a Theme:");

                    for section in self.themes.clone() {
                        if ui
                            .add(
                                egui::Button::new(&section)
                                    .min_size(Vec2 { x: 15.0, y: 10.0 })
                                    .fill(parse_color_from_ini("button-color")),
                            )
                            .clicked()
                        {
                            self.popup_open = false;
                            settings.set("settings", "current-theme", Some(section.to_string()));
                            let _ = settings.write(settings_path.clone());
                        }
                    }
                    if ui
                        .add(
                            egui::Button::new("close")
                                .min_size(Vec2 { x: 15.0, y: 10.0 })
                                .fill(parse_color_from_ini("button-color")),
                        )
                        .clicked()
                    {
                        self.popup_open = false;
                    }
                });
            });
        Ok(())
    }

    pub fn button_create(&mut self, ui: &mut egui::Ui) {
        let container_rect =
            egui::Rect::from_min_size(Pos2::new(430.0, 3.0), Vec2::new(15.0, 10.0));
        ui.allocate_ui_at_rect(container_rect, |ui| {
            if ui
                .add(
                    egui::Button::new("")
                        .min_size(Vec2 { x: 15.0, y: 10.0 })
                        .fill(parse_color_from_ini("button-color")),
                )
                .clicked()
            {
                self.popup_open = true;
            }
        });
    }
}
