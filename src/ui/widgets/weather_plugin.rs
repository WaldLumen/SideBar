use serde::Deserialize; // Importing serde for JSON deserialization
use reqwest;

// Struct to deserialize the JSON response from OpenWeatherMap API
#[derive(Deserialize, Debug)]
struct WeatherResponse {
    weather: Vec<Weather>, // Contains weather information
    main: Main, // Contains main weather parameters
    wind: Wind, // Contains wind information
    name: String, // Contains the name of the queried location
}

// Struct to represent weather description
#[derive(Deserialize, Debug)]
struct Weather {
    description: String, // Contains textual weather description
}

// Struct to represent main weather parameters
#[derive(Deserialize, Debug)]
struct Main {
    temp: f64, // Temperature in Celsius
    humidity: f64, // Humidity in percentage
    pressure: f64, // Atmospheric pressure in hPa
}

// Struct to represent wind information
#[derive(Deserialize, Debug)]
struct Wind {
    speed: f64, // Wind speed in meters per second
}

// Function to get weather information from OpenWeatherMap API
fn get_weather_info(city: &str, country_code: &str, api_key: &str) -> Result<WeatherResponse, reqwest::Error> {
    // Constructing the URL for API request
    let url = format!(
        "http://api.openweathermap.org/data/2.5/weather?q={},{}&units=metric&appid={}",
        city, country_code, api_key
    );

    // Sending a blocking GET request to the API endpoint
    let response = reqwest::blocking::get(&url)?;
    // Parsing the JSON response into WeatherResponse struct
    let response_json = response.json::<WeatherResponse>()?;
    Ok(response_json) // Returning the deserialized response
}


//    let description = &response.weather[0].description;
//    let temperature = response.main.temp;
//    let humidity = response.main.humidity;
//    let pressure = response.main.pressure;
//    let wind_speed = response.wind.speed;


// Function to get emoji based on temperature
fn get_temperature_emoji(temperature: f64) -> &'static str {
    if temperature < 0.0 {
        "❄️"
    } else if temperature >= 0.0 && temperature < 10.0 {
        "☁️"
    } else if temperature >= 10.0 && temperature < 20.0 {
        "⛅"
    } else if temperature >= 20.0 && temperature < 30.0 {
        "🌤️"
    } else {
        "🔥"
    }
}

pub fn get_weather() -> Vec<String>{
        let city = "Dnipro".trim();
        let country_code = "UA".trim();

        // Get your API key from OpenWeatherMap
        let api_key = "d8d31293bee740761c9ba933823c09ea"; 

        // Calling the function to fetch weather information
        match get_weather_info(&city, &country_code, api_key) {
            Ok(response) => {
                return vec![response.main.temp.to_string(),
			    response.weather[0].description.to_string(),
			    response.main.humidity.to_string(),
			    response.main.pressure.to_string(),
			    response.wind.speed.to_string()
		]; // Displaying weather information
            }
            Err(err) => {
                return vec![err.to_string()]; // Printing error message in case of failure
            }
        }
}
