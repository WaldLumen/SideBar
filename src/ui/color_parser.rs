use egui::Color32;
use std::env;
pub fn parse_color_from_ini(key: &str) -> Color32 {
    // Parse the INI contents using a simple approachlet
    let home: String = env::var("HOME").expect("Could not determine the home directory");
    let theme_path: String = format!("{}/{}", home, ".config/sidebar/themes.ini");
    let settings_path: String = format!("{}/{}", home, ".config/sidebar/settings.ini");
    let settings = ini!(&settings_path);
    let theme = ini!(&theme_path);
    let current_theme = settings["settings"]["current-theme"].clone().unwrap();
    let color_str = theme[&current_theme][key].clone().unwrap();

    // Parse the RGB values from the color string
    let rgb: Vec<u8> = color_str
        .split(',')
        .map(|v| v.trim().parse::<u8>())
        .collect::<Result<Vec<u8>, _>>()
        .expect("Failed to parse RGB values");

    // Ensure there are exactly 3 components (R, G, B)
    assert_eq!(rgb.len(), 3, "Expected exactly 3 RGB components");

    // Return the Color32 (assumes full opacity with alpha = 255)
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}
