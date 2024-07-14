use egui::{Frame, Pos2, Ui, Vec2,  TextStyle, FontId};
use crate::ui::widgets::weather_plugin::get_weather;

use chrono::{DateTime, Duration, Utc};

#[derive(Default)]
pub(crate) struct WeatherWidget {
    weather: Vec<String>,
    update_time: DateTime<Utc>,
    first_call: bool,
    emoji: String,
}

#[derive(Default)]
pub(crate) struct Weather {
}

impl WeatherWidget {
    fn get_weather_emoji(&mut self,) ->  String{
	if self.weather[1] == "clear sky"  {
            "".to_string()
	} else if self.weather[1] == "few clouds" {
            "".to_string()
	} else if self.weather[1] == "scattered clouds" {
            "".to_string()
	} else if self.weather[1] == "broken clouds" {
            "".to_string()
	} else if self.weather[1] == "shower rain" {
            "".to_string()
	} else if self.weather[1] == "rain" {
            "".to_string()
	} else if self.weather[1] == "thunderstorm" {
            "".to_string()
	} else if self.weather[1] == "snow" {
            "".to_string()
	} else if self.weather[1] == "mist" {
            "".to_string()
	} else {
	    "".to_string()
	}
    }
    
    pub fn show_weather_widget(&mut self, ui: &mut Ui) {
    let now = Utc::now();
    
    // Инициализация update_time при первом вызове функции
    if self.first_call {
        self.weather = get_weather();
        self.emoji = self.get_weather_emoji();
        self.update_time = now + Duration::minutes(5);
        self.first_call = false;
    } else if now >= self.update_time {
        self.weather = get_weather();
        self.emoji = self.get_weather_emoji();
        self.update_time = now + Duration::minutes(5);
    }
   
    // Создаем контейнер для виджета
    let container_size = Vec2::new(438.0, 50.0);
    
    let frame = Frame {
        fill: egui::Color32::from_rgb(255, 228, 225),
        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(253, 108, 158)),
        rounding: egui::Rounding::same(2.0),
        ..Default::default()
    };

    let custom_text_style = TextStyle::Name("custom".into());
    
    // Установка шрифта и размера для нового стиля текста
    ui.style_mut().text_styles.insert(
        custom_text_style.clone(),
        FontId::new(10.0, egui::FontFamily::Proportional),
    );

    // Создаем прямоугольник для размещения виджета
    let rect = egui::Rect::from_min_size(Pos2::new(7.0, 250.0), container_size);
    ui.allocate_ui_at_rect(rect, |ui| {
        frame.show(ui, |ui| {
            ui.allocate_space(Vec2::new(438.0, 5.0));
            
            let rect1 = egui::Rect::from_min_size(Pos2::new(14.0, 260.0), Vec2::new(90.0, 40.0));
            ui.allocate_ui_at_rect(rect1, |ui| {
                frame.show(ui, |ui| {
                    ui.allocate_space(Vec2::new(90.0, 3.0));
                    ui.vertical(|ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
			    ui.label(format!("{}", self.emoji));
			    ui.label(format!(":{:.2} °C", self.weather[0]));
                            ui.label(format!(":{} %", self.weather[2]));
                            ui.label(format!(":{} мм", self.weather[3]));
                            ui.label(format!(" :{} м/с", self.weather[4]));
                        });
                    });
                    ui.allocate_space(Vec2::new(0.0, 3.0));
                });
            });
	    ui.allocate_space(Vec2::new(438.0, 5.0));
        });
    });
}

}
