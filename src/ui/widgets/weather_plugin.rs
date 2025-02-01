use chrono::{DateTime, Duration, NaiveDate, Timelike, Utc};
use reqwest;
use serde::Deserialize; // Импортирование serde для десериализации JSON
use std::collections::HashMap;
use std::env;
use std::error::Error;

#[derive(Deserialize, Debug)]
pub struct WeatherForecast {
    pub list: Vec<WeatherEntry>, // Список прогнозов погоды
}

#[derive(Deserialize, Debug)]
pub struct WeatherEntry {
    pub dt: i64,                          // Метка времени прогноза
    pub main: Main,                       // Основные параметры погоды
    pub weather: Vec<WeatherDescription>, // Описание погоды
    pub wind: Wind,                       // Информация о ветре
}

#[derive(Deserialize, Debug)]
pub struct WeatherDescription {
    pub description: String, // Текстовое описание погоды
}

#[derive(Deserialize, Debug)]
pub struct Main {
    pub temp: f64,     // Температура в градусах Цельсия
    pub humidity: f64, // Влажность в процентах
    pub pressure: f64, // Атмосферное давление в гПа
}

#[derive(Deserialize, Debug)]
pub struct Wind {
    pub speed: f64, // Скорость ветра в метрах в секунду
}

fn get_weather_info(
    city: &str,
    country_code: &str,
    api_key: &str,
) -> Result<WeatherForecast, Box<dyn Error>> {
    let url = format!(
        "http://api.openweathermap.org/data/2.5/forecast?q={},{}&units=metric&appid={}",
        city, country_code, api_key
    );
    let response = reqwest::blocking::get(&url)?;
    let response_json = response.json::<WeatherForecast>()?;
    Ok(response_json)
}

pub fn get_weather() -> Result<WeatherForecast, Box<dyn Error>> {
    let home: String = env::var("HOME").expect("Could not determine the home directory");
    let settings_path: String = format!("{}/{}", home, ".config/sidebar/settings.ini");
    let settings = ini!(&settings_path);
    let city = settings["settings"]["city"].clone().unwrap();
    let country_code = settings["settings"]["country"].clone().unwrap();
    let api_key = settings["settings"]["owm_api_key"]
        .clone()
        .unwrap()
        .to_string();

    let forecast = get_weather_info(&city, &country_code, &api_key)?;

    // Текущая дата и время UTC
    let current_date = Utc::now();
    // Дата и время через 4 дня
    let end_date = current_date + Duration::days(4);

    // Группировка данных по дням и выбор записи для текущего дня и записи на час дня для следующих трех дней
    let mut daily_forecasts: HashMap<NaiveDate, WeatherEntry> = HashMap::new();

    for entry in forecast.list {
        let forecast_date =
            DateTime::<Utc>::from_utc(chrono::NaiveDateTime::from_timestamp(entry.dt, 0), Utc);
        let date = forecast_date.date().naive_utc();

        if date >= current_date.date().naive_utc() && date < end_date.date().naive_utc() {
            if date == current_date.date().naive_utc() {
                // Для текущего дня выбираем ближайшее к текущему времени значение
                daily_forecasts.entry(date).or_insert(entry);
            } else if forecast_date.hour() == 12 {
                // Для следующих дней выбираем значение на час дня
                daily_forecasts.entry(date).or_insert(entry);
            }
        }
    }

    let mut limited_forecast: Vec<WeatherEntry> = daily_forecasts
        .into_iter()
        .map(|(_, entry)| entry)
        .collect();

    // Сортировка записей по дате
    limited_forecast.sort_by_key(|entry| entry.dt);

    Ok(WeatherForecast {
        list: limited_forecast,
    })
}
