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
    
    // Weather settings
    city: String,
    country: String,
    api_key: String,
    
    // Health settings
    daily_water_goal: String,
    water_increment: String,
    daily_calorie_goal: String,
    
    settings_icon_texture: Option<egui::TextureHandle>,
    config_dir: Option<PathBuf>,
    theme_changed: bool,
    
    // UI state
    expanded_section: SettingsSection,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum SettingsSection {
    None,
    Themes,
    Weather,
    Health,
}

impl Default for SettingsSection {
    fn default() -> Self {
        Self::None
    }
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
        
        if !self.first_open {
            self.load_all_settings()?;
            self.first_open = true;
        }

        self.render_window(ctx);
        Ok(())
    }

    fn load_all_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let settings = self.load_ini("settings.ini")?;
        let themes = self.load_ini("themes.ini")?;
        
        // Load themes
        self.themes = themes.sections();
        
        // Load weather settings
        self.city = settings.get("settings", "city").unwrap_or_default();
        self.country = settings.get("settings", "country").unwrap_or_default();
        self.api_key = settings.get("settings", "owm_api_key").unwrap_or_default();
        
        // Load health settings with defaults
        self.daily_water_goal = settings
            .get("health", "daily_water_goal")
            .unwrap_or_else(|| "2000".to_string());
        self.water_increment = settings
            .get("health", "water_increment")
            .unwrap_or_else(|| "400".to_string());
        self.daily_calorie_goal = settings
            .get("health", "daily_calorie_goal")
            .unwrap_or_else(|| "2000".to_string());
        
        Ok(())
    }

    fn render_window(&mut self, ctx: &egui::Context) {
        let bg_color = parse_color_from_ini("background-color");
        let text_color = parse_color_from_ini("text-color");
        let accent_color = parse_color_from_ini("button-color");

        Window::new("⚙ Settings")
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .frame(egui::Frame::window(&ctx.style()).fill(bg_color))
            .show(ctx, |ui| {
                ui.style_mut().visuals.override_text_color = Some(text_color);
                ui.style_mut().visuals.widgets.inactive.bg_fill = accent_color.linear_multiply(0.3);
                ui.style_mut().visuals.widgets.hovered.bg_fill = accent_color.linear_multiply(0.5);
                ui.style_mut().visuals.widgets.active.bg_fill = accent_color;
                ui.style_mut().visuals.selection.bg_fill = accent_color.linear_multiply(0.7);
                
                egui::ScrollArea::vertical()
                    .id_source("settings_scroll")
                    .auto_shrink([false; 2])
                    .max_height(500.0)
                    .show(ui, |ui| {
                        self.render_theme_section(ui, ctx);
                        self.add_separator(ui);
                        self.render_health_section(ui);
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

    fn render_collapsible_header(&mut self, ui: &mut egui::Ui, section: SettingsSection, icon: &str, title: &str) -> bool {
        let is_expanded = self.expanded_section == section;
        let button_color = parse_color_from_ini("button-color");
        
        let arrow = if is_expanded { "▼" } else { "▶" };
        let response = ui.add(
            egui::Button::new(format!("{} {} {}", arrow, icon, title))
                .min_size(Vec2::new(ui.available_width(), 35.0))
                .fill(button_color.linear_multiply(if is_expanded { 0.8 } else { 0.5 }))
        );
        
        if response.clicked() {
            self.expanded_section = if is_expanded { 
                SettingsSection::None 
            } else { 
                section 
            };
        }
        
        is_expanded
    }

    fn render_theme_section(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.render_collapsible_header(ui, SettingsSection::Themes, "🎨", "Themes") {
            ui.add_space(10.0);
            
            let current_theme = self.get_current_theme();
            let mut selected_theme = current_theme.clone();

            ui.horizontal(|ui| {
                ui.label("Select theme:");
                egui::ComboBox::from_label("")
                    .selected_text(&selected_theme)
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        for theme in &self.themes {
                            ui.selectable_value(&mut selected_theme, theme.clone(), theme);
                        }
                    });
            });

            if selected_theme != current_theme {
                self.apply_theme(&selected_theme);
                self.theme_changed = true;
                ctx.request_repaint();
            }
            
            ui.add_space(10.0);
        }
    }

    fn render_health_section(&mut self, ui: &mut egui::Ui) {
        if self.render_collapsible_header(ui, SettingsSection::Health, "💪", "Health Tracking") {
            ui.add_space(10.0);
            
            ui.heading("Water Settings");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Daily goal (ml):");
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.daily_water_goal)
                        .desired_width(100.0)
                        .hint_text("2000")
                );
            });
            ui.add_space(3.0);
            
            ui.horizontal(|ui| {
                ui.label("Increment (ml):");
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.water_increment)
                        .desired_width(100.0)
                        .hint_text("400")
                );
            });
            
            ui.add_space(15.0);
            ui.heading("Food Settings");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.label("Daily goal (kcal):");
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.daily_calorie_goal)
                        .desired_width(100.0)
                        .hint_text("2000")
                );
            });
            
            ui.add_space(10.0);
            
            if ui.add(
                egui::Button::new("💾 Save Health Settings")
                    .min_size(Vec2::new(200.0, 30.0))
                    .fill(parse_color_from_ini("button-color"))
            ).clicked() {
                self.save_health_settings();
            }
            
            ui.add_space(10.0);
        }
    }

    fn save_health_settings(&self) {
        if let Ok(mut settings) = self.load_ini("settings.ini") {
            // Validate and save water goal
            if let Ok(goal) = self.daily_water_goal.parse::<u32>() {
                settings.set("health", "daily_water_goal", Some(goal.to_string()));
            }
            
            // Validate and save water increment
            if let Ok(increment) = self.water_increment.parse::<u32>() {
                settings.set("health", "water_increment", Some(increment.to_string()));
            }
            
            // Validate and save calorie goal
            if let Ok(goal) = self.daily_calorie_goal.parse::<i32>() {
                settings.set("health", "daily_calorie_goal", Some(goal.to_string()));
            }
            
            let _ = self.save_ini(&settings, "settings.ini");
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
            invalidate_color_cache();
        }
    }

    fn render_weather_settings(&mut self, ui: &mut egui::Ui) {
        if self.render_collapsible_header(ui, SettingsSection::Weather, "🌤", "Weather Settings") {
            ui.add_space(10.0);

            render_text_input(ui, "City:", &mut self.city);
            render_text_input(ui, "Country:", &mut self.country);
            render_text_input(ui, "OpenWeatherMap API Key:", &mut self.api_key);

            if ui.add(
                egui::Button::new("💾 Save Weather Settings")
                    .min_size(Vec2::new(200.0, 30.0))
                    .fill(parse_color_from_ini("button-color")),
            ).clicked() {
                self.save_weather_settings();
            }
            
            ui.add_space(10.0);
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
            if ui.add(
                egui::Button::new("✖ Close")
                    .min_size(Vec2::new(100.0, 30.0))
                    .fill(parse_color_from_ini("button-color")),
            ).clicked() {
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

pub fn get_daily_water_goal() -> u32 {
    get_health_setting("daily_water_goal", "2000")
        .parse()
        .unwrap_or(2000)
}

pub fn get_water_increment() -> u32 {
    get_health_setting("water_increment", "400")
        .parse()
        .unwrap_or(400)
}

pub fn get_daily_calorie_goal() -> i32 {
    get_health_setting("daily_calorie_goal", "2000")
        .parse()
        .unwrap_or(2000)
}

fn get_health_setting(key: &str, default: &str) -> String {
    let config_dir = match env::var("HOME") {
        Ok(home) => PathBuf::from(home).join(".config/sidebar"),
        Err(_) => return default.to_string(),
    };
    
    let mut ini = Ini::new();
    let settings_path = config_dir.join("settings.ini");
    
    if ini.load(&settings_path).is_ok() {
        ini.get("health", key).unwrap_or_else(|| default.to_string())
    } else {
        default.to_string()
    }
}