use crate::ui::color_parser::parse_color_from_ini;
use configparser::ini::Ini;
use egui::{Pos2, Vec2, Window};
use std::env;
use std::error::Error;
use image::GenericImageView;
use crate::ui::custom_vidgets::StyledImageButton;

#[derive(Default)]
pub struct Settings {
    first_open: bool,
    themes: Vec<String>,
    pub popup_open: bool,
    city: String,
    country: String,
    api_key: String,
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

        // Читаем настройки
        self.city = settings.get("settings", "city").unwrap().to_string();
        self.country = settings.get("settings", "country").unwrap().to_string();
        self.api_key = settings.get("settings", "owm_api_key").unwrap().to_string();

        if !self.first_open {
            self.themes = themes.sections();
            self.first_open = true;
        }

        Window::new("Edit Theme")
            .title_bar(false)
            .default_pos(egui::pos2(75.0, 200.0))
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::Vec2::new(300.0, 500.0))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label("Settings:");
                    ui.separator();
                    ui.label("Themes:");
                    ui.horizontal(|ui| {
                        for section in self.themes.clone() {
                            if ui.button(&section).clicked() {
                                self.popup_open = false;
                                settings.set(
                                    "settings",
                                    "current-theme",
                                    Some(section.to_string()),
                                );
                                let _ = settings.write(settings_path.clone());
                            }
                        }
                    });

                    ui.separator();

                    let mut city_input = self.city.clone();
                    let mut country_input = self.country.clone();
                    let mut api_key_input = self.api_key.clone();

                    ui.label("City:");
                    ui.text_edit_singleline(&mut city_input);
                    ui.label("Country:");
                    ui.text_edit_singleline(&mut country_input);
                    ui.label("OpenWeatherMap API Key:");
                    ui.text_edit_singleline(&mut api_key_input);

                    if ui.button("Save Settings").clicked() {
                        self.city = city_input.clone();
                        self.country = country_input.clone();
                        self.api_key = api_key_input.clone();

                        settings.set("settings", "city", Some(city_input));
                        settings.set("settings", "country", Some(country_input));
                        settings.set("settings", "api_key", Some(api_key_input));
                        let _ = settings.write(settings_path.clone());
                    }

                    if ui.button("Close").clicked() {
                        self.popup_open = false;
                    }
                });
            });
        Ok(())
    }
    pub fn button_create(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
    let img = image::open("/home/rika/code/SideBar-Rust/src/assets/icons/settings.png").unwrap();
    let size = [img.width() as usize, img.height() as usize];
    let image_buffer = img.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    let texture = ctx.load_texture("my_image", color_image, egui::TextureOptions::default());
    let container_rect = egui::Rect::from_min_size(Pos2::new(410.2, 8.0), Vec2::new(15.0, 10.0));
    
    ui.allocate_ui_at_rect(container_rect, |ui| {
        if StyledImageButton::new(&texture)
            .size(Vec2::new(17.0, 17.0))
            .bg_color(parse_color_from_ini("button-color"))
            .rounding(4.0)
            .show(ui)
            .clicked()
        {
            self.popup_open = true;
        }
    });
}
}
