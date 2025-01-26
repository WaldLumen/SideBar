use crate::ui::widgets::weather_plugin::{get_weather, WeatherForecast};
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use egui::{FontId, Frame, Pos2, TextStyle, Ui, Vec2};

#[derive(Default)]
pub(crate) struct WeatherWidget {
    weather_forecast: Option<WeatherForecast>,
    update_time: DateTime<Utc>,
    first_call: bool,
    emoji_list: Vec<String>,
}

impl WeatherWidget {
    fn get_day_of_week(&self, dt: i64) -> String {
        let naive_datetime = NaiveDateTime::from_timestamp(dt, 0);
        let datetime: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);
        datetime.format("%A").to_string()
    }

    fn get_weather_emoji(&self, description: &str) -> String {
        match description {
            "clear sky" => "".to_string(),
            "few clouds" => "".to_string(),
            "scattered clouds" => "".to_string(),
            "broken clouds" => "".to_string(),
            "overcast clouds" => "".to_string(),
            "shower rain" => "".to_string(),
            "rain" => "".to_string(),
            "thunderstorm" => "".to_string(),
            "snow" => "".to_string(),
            "mist" => "".to_string(),
            _ => description.to_string(),
        }
    }

    pub fn show_weather_widget(&mut self, ui: &mut Ui) {
        let now = Utc::now();

        if self.first_call || now >= self.update_time {
            match get_weather() {
                Ok(weather_forecast) => {
                    self.weather_forecast = Some(weather_forecast);
                    self.emoji_list = self
                        .weather_forecast
                        .as_ref()
                        .unwrap()
                        .list
                        .iter()
                        .map(|entry| self.get_weather_emoji(&entry.weather[0].description))
                        .collect();
                }
                Err(err) => {
                    println!("Error fetching weather data: {}", err);
                }
            }
            self.update_time = now + Duration::minutes(5);
            self.first_call = false;
        }

        if let Some(weather_forecast) = &self.weather_forecast {
            let container_size = Vec2::new(438.0, 50.0);
            let frame = Frame {
                rounding: egui::Rounding::same(2.0),
                ..Default::default()
            };

            let custom_text_style = TextStyle::Name("custom".into());
            ui.style_mut().text_styles.insert(
                custom_text_style.clone(),
                FontId::new(10.0, egui::FontFamily::Proportional),
            );

            let rect = egui::Rect::from_min_size(Pos2::new(7.0, 250.0), container_size);
            ui.allocate_ui_at_rect(rect, |ui| {
                frame.show(ui, |ui| {
                    ui.allocate_space(Vec2::new(438.0, 5.0));

                    for (i, entry) in weather_forecast.list.iter().take(4).enumerate() {
                        let rect1 = egui::Rect::from_min_size(
                            Pos2::new(14.0 + i as f32 * 110.0, 260.0),
                            Vec2::new(90.0, 40.0),
                        );
                        ui.allocate_ui_at_rect(rect1, |ui| {
                            frame.show(ui, |ui| {
                                ui.allocate_space(Vec2::new(90.0, 3.0));
                                ui.vertical(|ui| {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
                                            let day_of_week = self.get_day_of_week(entry.dt);
                                            ui.label(format!("{}:", day_of_week));
                                            ui.label(format!("{}", self.emoji_list[i]));
                                            ui.label(format!(": {}°C", entry.main.temp as i32));
                                            ui.label(format!(": {}%", entry.main.humidity));
                                            ui.label(format!(": {}мм", entry.main.pressure));
                                            ui.label(format!(": {}м/с", entry.wind.speed));
                                        },
                                    );
                                });
                                ui.allocate_space(Vec2::new(0.0, 3.0));
                            });
                        });
                    }
                    ui.allocate_space(Vec2::new(438.0, 5.0));
                });
            });
        }
    }
}
