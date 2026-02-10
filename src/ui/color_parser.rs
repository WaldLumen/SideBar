use egui::Color32;
use std::env;
use std::sync::RwLock;
use std::collections::HashMap;
use once_cell::sync::Lazy;

// Глобальный кэш цветов
static COLOR_CACHE: Lazy<RwLock<HashMap<String, Color32>>> = 
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn parse_color_from_ini(key: &str) -> Color32 {
    // Проверяем кэш
    if let Ok(cache) = COLOR_CACHE.read() {
        if let Some(&color) = cache.get(key) {
            return color;
        }
    }
    
    // Парсим цвет
    let color = load_color_from_file(key);
    
    // Сохраняем в кэш
    if let Ok(mut cache) = COLOR_CACHE.write() {
        cache.insert(key.to_string(), color);
    }
    
    color
}

fn load_color_from_file(key: &str) -> Color32 {
    let home = match env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Color32::WHITE,
    };
    
    let theme_path = format!("{}/.config/sidebar/themes.ini", home);
    let settings_path = format!("{}/.config/sidebar/settings.ini", home);
    
    let settings = ini!(&settings_path);
    let theme = ini!(&theme_path);
    
    // Исправлено: используем map_or для получения &str
    let current_theme = settings
        .get("settings")
        .and_then(|s| s.get("current-theme"))
        .and_then(|t| t.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("yellow");
    
    // Исправлено: используем map для получения &str
    let color_str = theme
        .get(current_theme)
        .and_then(|t| t.get(key))
        .and_then(|c| c.as_ref())
        .map(|s| s.as_str())
        .unwrap_or("255, 255, 255");
    
    parse_rgb_string(color_str)
}

fn parse_rgb_string(s: &str) -> Color32 {
    let parts: Vec<&str> = s.split(',').collect();
    
    match parts.len() {
        3 => {
            let r = parts[0].trim().parse().unwrap_or(255);
            let g = parts[1].trim().parse().unwrap_or(255);
            let b = parts[2].trim().parse().unwrap_or(255);
            Color32::from_rgb(r, g, b)
        }
        4 => {
            let r = parts[0].trim().parse().unwrap_or(255);
            let g = parts[1].trim().parse().unwrap_or(255);
            let b = parts[2].trim().parse().unwrap_or(255);
            let a = parts[3].trim().parse().unwrap_or(255);
            Color32::from_rgba_unmultiplied(r, g, b, a)
        }
        _ => Color32::WHITE,
    }
}

// Вызывать при смене темы
pub fn invalidate_color_cache() {
    if let Ok(mut cache) = COLOR_CACHE.write() {
        cache.clear();
    }
}