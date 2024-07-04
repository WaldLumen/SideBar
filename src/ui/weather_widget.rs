use egui::{Ui, Vec2, Frame, Pos2};
use crate::ui::widgets::weather_plugin::get_weather;

#[derive(Default)]
pub(crate) struct WeatherWidget {
}

#[derive(Default)]
pub(crate) struct Weather {
}

impl WeatherWidget {
    pub fn show_weather_widget(&mut self, ui: &mut Ui) {
        // Логика отображения погоды
        let weather = get_weather();


	let container_size = Vec2::new(430.0, 24.0);
        let frame = Frame {
            fill: egui::Color32::from_rgb(255, 228, 225),
            stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(253, 108, 158)),
            rounding: egui::Rounding::same(2.0),
            ..Default::default()
        };

	let rect = egui::Rect::from_min_size(Pos2::new(7.0, 250.0), Vec2::new(430.0, 24.0));
        ui.allocate_ui_at_rect(rect, |ui| {
	    frame.show(ui, |ui| {
		ui.set_min_size(container_size);
		ui.allocate_space(Vec2::new(430.0, 1.0));
		ui.label("                          Temp:");
		ui.label(format!("            {} {}", weather[0].to_string(), weather[1].to_string()));
            });
        });
    }
}
