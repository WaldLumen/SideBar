use crate::ui::color_parser::parse_color_from_ini;
use crate::ui::custom_vidgets::StyledImageButton;
use configparser::ini::Ini;
use egui::{Vec2, Window};
use image::GenericImageView;
use std::env;
use std::error::Error;

#[derive(Default)]
pub struct Settings {
    first_open: bool,
    themes: Vec<String>,
    pub popup_open: bool,
    city: String,
    country: String,
    api_key: String,
    settings_icon_texture: Option<egui::TextureHandle>,
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
        self.city = settings.get("settings", "city").unwrap_or_default();
        self.country = settings.get("settings", "country").unwrap_or_default();
        self.api_key = settings
            .get("settings", "owm_api_key")
            .unwrap_or_default();

        if !self.first_open {
            self.themes = themes.sections();
            self.first_open = true;
        }

        Window::new("Settings")
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .default_width(350.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.render_theme_section(ui, &settings_path);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        self.render_weather_settings(ui, &settings_path);
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);
                        self.render_action_buttons(ui);
                    });
            });
        Ok(())
    }

    fn render_theme_section(&mut self, ui: &mut egui::Ui, settings_path: &str) {
        ui.heading("🎨 Themes");
        ui.add_space(5.0);

        ui.horizontal_wrapped(|ui| {
            for theme in self.themes.clone() {
                if ui
                    .add(
                        egui::Button::new(&theme)
                            .min_size(Vec2::new(80.0, 30.0))
                            .fill(parse_color_from_ini("button-color")),
                    )
                    .clicked()
                {
                    self.apply_theme(&theme, settings_path);
                }
            }
        });
    }

    fn apply_theme(&mut self, theme: &str, settings_path: &str) {
        let mut settings = Ini::new();
        if settings.load(settings_path).is_ok() {
            settings.set("settings", "current-theme", Some(theme.to_string()));
            let _ = settings.write(settings_path);
            self.popup_open = false;
        }
    }

    fn render_weather_settings(&mut self, ui: &mut egui::Ui, settings_path: &str) {
        ui.heading("🌤 Weather Settings");
        ui.add_space(5.0);

        ui.label("City:");
        ui.text_edit_singleline(&mut self.city);
        ui.add_space(5.0);

        ui.label("Country:");
        ui.text_edit_singleline(&mut self.country);
        ui.add_space(5.0);

        ui.label("OpenWeatherMap API Key:");
        ui.text_edit_singleline(&mut self.api_key);
        ui.add_space(5.0);

        if ui
            .add(
                egui::Button::new("💾 Save Weather Settings")
                    .min_size(Vec2::new(200.0, 30.0))
                    .fill(parse_color_from_ini("button-color")),
            )
            .clicked()
        {
            self.save_weather_settings(settings_path);
        }
    }

    fn save_weather_settings(&self, settings_path: &str) {
        let mut settings = Ini::new();
        if settings.load(settings_path).is_ok() {
            settings.set("settings", "city", Some(self.city.clone()));
            settings.set("settings", "country", Some(self.country.clone()));
            settings.set("settings", "owm_api_key", Some(self.api_key.clone()));
            let _ = settings.write(settings_path);
        }
    }

    fn render_action_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new("✖ Close")
                        .min_size(Vec2::new(100.0, 30.0))
                        .fill(parse_color_from_ini("button-color")),
                )
                .clicked()
            {
                self.popup_open = false;
            }
        });
    }

    pub fn button_create(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // Загружаем текстуру один раз
        self.load_texture_if_needed(ctx);

        if let Some(texture) = &self.settings_icon_texture {
            if StyledImageButton::new(texture)
                .size(Vec2::new(22.0, 22.0))
                .bg_color(parse_color_from_ini("button-color"))
                .rounding(6.0)
                .show(ui)
                .on_hover_text("Settings")
                .clicked()
            {
                self.popup_open = true;
            }
        }
    }

    fn load_texture_if_needed(&mut self, ctx: &egui::Context) {
        if self.settings_icon_texture.is_none() {
            if let Ok(img) =
                image::open("/home/rika/code/SideBar-Rust/src/assets/icons/settings.png")
            {
                let size = [img.width() as usize, img.height() as usize];
                let image_buffer = img.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image =
                    egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                let texture = ctx.load_texture(
                    "settings_icon",
                    color_image,
                    egui::TextureOptions::default(),
                );
                self.settings_icon_texture = Some(texture);
            }
        }
    }
}