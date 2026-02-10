use crate::ui::color_parser::{parse_color_from_ini, invalidate_color_cache};
use crate::ui::custom_vidgets::StyledImageButton;
use configparser::ini::Ini;
use egui::{Vec2, Window};
use std::env;
use std::error::Error;
use std::path::PathBuf;

#[derive(Default)]
pub struct Settings {
    first_open: bool,
    themes: Vec<String>,
    pub popup_open: bool,
    city: String,
    country: String,
    api_key: String,
    settings_icon_texture: Option<egui::TextureHandle>,
    config_dir: Option<PathBuf>,
    theme_changed: bool, // Флаг для отслеживания смены темы
}

impl Settings {
    fn get_config_dir() -> Result<PathBuf, Box<dyn Error>> {
        let home = env::var("HOME")?;
        Ok(PathBuf::from(home).join(".config/sidebar"))
    }

    fn ensure_config_dir(&mut self) -> Result<&PathBuf, Box<dyn Error>> {
        if self.config_dir.is_none() {
            self.config_dir = Some(Self::get_config_dir()?);
        }
        Ok(self.config_dir.as_ref().unwrap())
    }

    fn load_ini(&self, filename: &str) -> Result<Ini, Box<dyn Error>> {
        let mut ini = Ini::new();
        let path = self.config_dir.as_ref()
            .ok_or("Config directory not initialized")?
            .join(filename);
        ini.load(path)?;
        Ok(ini)
    }

    fn save_ini(&self, ini: &Ini, filename: &str) -> Result<(), Box<dyn Error>> {
        let path = self.config_dir.as_ref()
            .ok_or("Config directory not initialized")?
            .join(filename);
        ini.write(path)?;
        Ok(())
    }

    pub fn create_settings_window(&mut self, ctx: &egui::Context) -> Result<(), Box<dyn Error>> {
        self.ensure_config_dir()?;
        
        let settings = self.load_ini("settings.ini")?;
        
        if !self.first_open {
            let themes = self.load_ini("themes.ini")?;
            self.themes = themes.sections();
            self.city = settings.get("settings", "city").unwrap_or_default();
            self.country = settings.get("settings", "country").unwrap_or_default();
            self.api_key = settings.get("settings", "owm_api_key").unwrap_or_default();
            self.first_open = true;
        }

        self.render_window(ctx);
        Ok(())
    }

    fn render_window(&mut self, ctx: &egui::Context) {
        let bg_color = parse_color_from_ini("background-color");
        let text_color = parse_color_from_ini("text-color");
        let accent_color = parse_color_from_ini("button-color");

        Window::new("Settings")
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .default_width(350.0)
            .frame(egui::Frame::window(&ctx.style()).fill(bg_color))
            .show(ctx, |ui| {
                ui.style_mut().visuals.override_text_color = Some(text_color);
                ui.style_mut().visuals.widgets.inactive.bg_fill = accent_color.linear_multiply(0.3);
                ui.style_mut().visuals.widgets.hovered.bg_fill = accent_color.linear_multiply(0.5);
                ui.style_mut().visuals.widgets.active.bg_fill = accent_color;
                ui.style_mut().visuals.selection.bg_fill = accent_color.linear_multiply(0.7);
                
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        self.render_theme_section(ui, ctx);
                        self.add_separator(ui);
                        self.render_weather_settings(ui);
                        self.add_separator(ui);
                        self.render_action_buttons(ui);
                    });
            });
    }

    fn add_separator(&self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
    }

    fn render_theme_section(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("🎨 Themes");
        ui.add_space(5.0);

        let current_theme = self.get_current_theme();
        let mut selected_theme = current_theme.clone();

        ui.horizontal(|ui| {
            ui.label("Select theme:");
            egui::ComboBox::from_label("")
                .selected_text(&selected_theme)
                .width(200.0)
                .show_ui(ui, |ui| {
                    for theme in &self.themes {
                        ui.selectable_value(&mut selected_theme, theme.clone(), theme);
                    }
                });
        });

        // Применяем тему если она изменилась
        if selected_theme != current_theme {
            self.apply_theme(&selected_theme);
            self.theme_changed = true;
            ctx.request_repaint(); // Принудительная перерисовка
        }
    }

    fn get_current_theme(&self) -> String {
        self.load_ini("settings.ini")
            .ok()
            .and_then(|s| s.get("settings", "current-theme"))
            .unwrap_or_else(|| "yellow".to_string())
    }

    fn apply_theme(&mut self, theme: &str) {
        if let Ok(mut settings) = self.load_ini("settings.ini") {
            settings.set("settings", "current-theme", Some(theme.to_string()));
            let _ = self.save_ini(&settings, "settings.ini");
            
            // КРИТИЧНО: очищаем кэш цветов после смены темы
            invalidate_color_cache();
        }
    }

    fn render_weather_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("🌤 Weather Settings");
        ui.add_space(5.0);

        render_text_input(ui, "City:", &mut self.city);
        render_text_input(ui, "Country:", &mut self.country);
        render_text_input(ui, "OpenWeatherMap API Key:", &mut self.api_key);

        if ui
            .add(
                egui::Button::new("💾 Save Weather Settings")
                    .min_size(Vec2::new(200.0, 30.0))
                    .fill(parse_color_from_ini("button-color")),
            )
            .clicked()
        {
            self.save_weather_settings();
        }
    }

    fn save_weather_settings(&self) {
        if let Ok(mut settings) = self.load_ini("settings.ini") {
            settings.set("settings", "city", Some(self.city.clone()));
            settings.set("settings", "country", Some(self.country.clone()));
            settings.set("settings", "owm_api_key", Some(self.api_key.clone()));
            let _ = self.save_ini(&settings, "settings.ini");
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
            let icon_path = if let Ok(config_dir) = Self::get_config_dir() {
                config_dir.parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("code/SideBar-Rust/src/assets/icons/settings.png"))
                    .unwrap_or_else(|| PathBuf::from("src/assets/icons/settings.png"))
            } else {
                PathBuf::from("src/assets/icons/settings.png")
            };

            if let Ok(img) = image::open(&icon_path) {
                let size = [img.width() as usize, img.height() as usize];
                let image_buffer = img.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                
                self.settings_icon_texture = Some(ctx.load_texture(
                    "settings_icon",
                    color_image,
                    egui::TextureOptions::default(),
                ));
            }
        }
    }
    
    // Проверка, была ли изменена тема
    pub fn was_theme_changed(&mut self) -> bool {
        let changed = self.theme_changed;
        self.theme_changed = false;
        changed
    }
}

fn render_text_input(ui: &mut egui::Ui, label: &str, text: &mut String) {
    ui.label(label);
    let input_bg = parse_color_from_ini("background-color").linear_multiply(1.2);
    ui.style_mut().visuals.extreme_bg_color = input_bg;
    ui.text_edit_singleline(text);
    ui.add_space(5.0);
}