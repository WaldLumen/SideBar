use crate::ui::color_parser::parse_color_from_ini;
use bincode;
use calory_fetch::{fetch_calory_of_certain_food, fetch_data};
use chrono::{DateTime, Local};
use egui::{Frame, Pos2, TextEdit, Ui, Vec2, Window};
use sled::{Db, Result};
use std::env;

use tokio::runtime::Runtime;

#[derive(Default)]
pub struct FoodWidget {
    pub query: String,
    pub dish_name: String,
    pub results: Vec<(String, String)>,
    pub dish_calory: i32, // Хранит список (название и URL) для каждого результата
    pub selected_food: Option<(String, String)>, // Хранит выбранное название и калорийность блюда
    pub db: Option<Db>,   // Опциональная база данных
    pub calory: i32,      // Калорийность для текущей даты
    pub calory_popup: bool,
    pub food_amount: String,
}

impl FoodWidget {
    fn open_db(&self, path: &str) -> Result<Db> {
        sled::open(path)
    }
    pub fn calory_popup(&mut self, ctx: &egui::Context) {
        if self.calory_popup {
            Window::new("Calory")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .fixed_size(Vec2::new(300.0, 500.0))
                .show(ctx, |ui| {
                    // Label for input
                    ui.label(self.dish_name.clone());
                    ui.label(format!("{} kkal in 100 mg", self.dish_calory));
                    ui.label("Food Amount (in grams):");
                    // Input field with hint text
                    ui.add(
                        TextEdit::multiline(&mut self.food_amount)
                            .hint_text("Enter amount here...")
                            .min_size(Vec2::new(300.0, 100.0)),
                    );

                    // Parse food_amount input
                    if let Ok(food_amount_int) = self.food_amount.parse::<i32>() {
                        let calculated_calory = food_amount_int * self.dish_calory / 100;

                        // Display calculated calories
                        ui.label(format!("Calories: {}", calculated_calory));

                        // Add button (only active when input is valid)
                        if ui.button("Add").clicked() {
                            self.update_today_calory(calculated_calory);
                            self.calory_popup = false;
                        }
                    }

                    /*
                    Close button
                    */
                    if ui.button("Close").clicked() {
                        self.calory_popup = false;
                    }
                });
        }
    }

    fn init_today_record(&mut self) {
        if let Some(db) = &self.db {
            let date = self.get_date();
            if let Some(stored_value) = db.get(date.as_bytes()).unwrap() {
                self.calory = bincode::deserialize(&stored_value).unwrap();
                println!(
                    "Запись на дату {} уже существует, значение: {} калорий",
                    date, self.calory
                );
            } else {
                self.calory = 0;
                db.insert(date.as_bytes(), bincode::serialize(&self.calory).unwrap())
                    .unwrap();
                db.flush().unwrap();
                println!(
                    "Создана новая запись для {} с начальным значением: 0 калорий",
                    date
                );
            }
        }
    }

    fn update_today_calory(&mut self, additional_calory: i32) {
        if let Some(db) = &self.db {
            let date = self.get_date();
            self.calory += additional_calory;
            db.insert(date.as_bytes(), bincode::serialize(&self.calory).unwrap())
                .unwrap();
            db.flush().unwrap();
            println!(
                "Обновлено значение калорий на дату {}: {} калорий",
                date, self.calory
            );
        }
    }

    fn calorie_adjustment_widget(&mut self, ui: &mut Ui, frame: Frame) -> Result<()> {
        let home: String = env::var("HOME").expect("Could not determine the home directory");
        let db_path: String = format!("{}/{}", home, ".local/share/SideBarFoodDb");
        let rt = Runtime::new().unwrap();
        let date = self.get_date();
        if self.db.is_none() {
            self.db = Some(self.open_db(&db_path)?);
            self.init_today_record();
        }

        let container_rect =
            egui::Rect::from_min_size(Pos2::new(110.0, 420.0), Vec2::new(90.0, 60.0));
        ui.allocate_ui_at_rect(container_rect, |ui| {
            ui.allocate_space(Vec2::new(0.0, 3.0));
            frame.show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.allocate_space(Vec2::new(0.0, 6.0));
                    ui.label("󰉚");
                    ui.label(date.clone());
                    ui.label(format!("{} kkal", self.calory));
                    ui.label("2000 kkal");
                    ui.allocate_space(Vec2::new(0.0, 3.0));
                });

                let search_rect =
                    egui::Rect::from_min_size(Pos2::new(205.0, 433.0), Vec2::new(340.0, 120.0));
                ui.allocate_ui_at_rect(search_rect, |ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // Поле для ввода количества калорий для добавления или вычитания

                            let add_sized = ui.add_sized(
                                Vec2::new(135.0, 25.0),
                                egui::TextEdit::singleline(&mut self.query),
                            );

                            if ui
                                .add_sized(
                                    Vec2::new(80.0, 25.0),
                                    egui::Button::new("Search")
                                        .fill(parse_color_from_ini("button-color")),
                                )
                                .clicked()
                            {
                                let result = rt.block_on(fetch_data(&self.query));
                                self.results = result
                                    .into_iter()
                                    .map(|item| (item.value.clone(), item.url.clone()))
                                    .collect();
                            }
                            ui.allocate_space(Vec2::new(3.0, 3.0));
                        });

                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .max_width(230.0)
                            .max_height(13.0)
                            .id_source("food_results_scroll_area")
                            .show(ui, |ui| {
                                for (name, url) in self.results.clone() {
                                    if ui
                                        .add_sized(
                                            Vec2::new(ui.available_width(), 30.0),
                                            egui::Button::new(name.clone())
                                                .fill(parse_color_from_ini("button-color")),
                                        )
                                        .clicked()
                                    {
                                        let calory =
                                            rt.block_on(fetch_calory_of_certain_food(url.clone()));

                                        // Сохраняем выбранное блюдо
                                        self.selected_food =
                                            Some((calory[1].clone(), calory[0].clone()));

                                        self.dish_calory = calory[0]
                                            .clone()
                                            .chars()
                                            .filter(|c| c.is_ascii_digit()) // Keep only numeric characters
                                            .collect::<String>() // Collect into a String
                                            .parse() // Parse the String to an i32
                                            .unwrap_or(0); // Default to 0 if parsing fails

                                        // Обновляем данные калорийности вне замыкания

                                        self.food_amount = "0".to_string();
                                        self.calory_popup = true;
                                    }
                                }
                            });
                    });
                });
            });
        });
        Ok(())
    }

    fn get_date(&self) -> String {
        let local: DateTime<Local> = Local::now();
        local.format("%d.%m").to_string()
    }
}

#[derive(Default)]
pub struct WaterManager {
    pub water_amount: u32,
    pub is_update: bool,
    pub is_first_call: bool,
    pub is_first_init: bool,
}

impl WaterManager {
    pub fn water_widget(&mut self, ui: &mut egui::Ui, frame: Frame) -> Result<()> {
        let date: String = self.get_date();
        let home: String = env::var("HOME").expect("Could not determine the home directory");
        let db_path: String = format!("{}/{}", home, ".local/share/SideBarWaterDb");

        // Открываем базу данных с обработкой ошибок
        let db: Db = self.open_db(&db_path)?;
        if !self.is_first_init {
            // Инициализируем запись на текущую дату и устанавливаем значение self.water_amount
            self.init_today_record(&db, date.clone())?;
            self.is_first_init = true;
        }
        if self.is_update || self.is_first_call {
            match self.get_water_data(&db, date.clone()).unwrap() {
                Some(amount) => {
                    let rect1 =
                        egui::Rect::from_min_size(Pos2::new(14.0, 413.0), Vec2::new(90.0, 40.0));

                    ui.allocate_space(Vec2::new(438.0, 1.0));

                    ui.allocate_ui_at_rect(rect1, |ui| {
                        ui.allocate_space(Vec2::new(1.0, 3.0));
                        frame.show(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.allocate_space(Vec2::new(1.0, 3.0));
                                ui.label("󰆫");
                                ui.label(date.clone().to_string());
                                ui.label(format!("{} ml", amount));
                                ui.label("2000 ml");
                                ui.horizontal(|ui| {
                                    ui.allocate_space(Vec2::new(15.0, 1.0));
                                    if ui
                                        .add(
                                            egui::Button::new("+")
                                                .min_size(Vec2 { x: 14.0, y: 10.0 })
                                                .fill(parse_color_from_ini("button-color")),
                                        )
                                        .clicked()
                                    {
                                        self.water_amount += 400;
                                        let _ = self.update_water_data(
                                            &db,
                                            date.clone(),
                                            self.water_amount,
                                        );
                                        self.is_update = true;
                                    }
                                    if ui
                                        .add(
                                            egui::Button::new("-")
                                                .fill(parse_color_from_ini("button-color"))
                                                .min_size(Vec2 { x: 14.0, y: 10.0 }),
                                        )
                                        .clicked()
                                    {
                                        self.water_amount = self.water_amount.saturating_sub(400);
                                        let _ = self.update_water_data(
                                            &db,
                                            date.clone(),
                                            self.water_amount,
                                        );
                                        self.is_update = true;
                                    }
                                });
                                ui.allocate_space(Vec2::new(1.0, 3.0));
                            });
                        });
                        ui.allocate_space(Vec2::new(1.0, 3.0));
                    });
                }
                None => {
                    ui.label(format!("Запись для {} не найдена.", date));
                }
            }
            self.is_first_call = false;
        }

        Ok(())
    }

    fn open_db(&mut self, path: &str) -> Result<Db> {
        let db = sled::open(path)?;
        Ok(db)
    }

    fn get_date(&mut self) -> String {
        let local: DateTime<Local> = Local::now();
        local.format("%d.%m").to_string()
    }

    fn init_today_record(&mut self, db: &Db, date: String) -> sled::Result<()> {
        // Получаем данные из базы для текущей даты
        if let Some(stored_value) = db.get(date.as_bytes())? {
            // Если запись существует, загружаем количество воды
            self.water_amount = bincode::deserialize(&stored_value).unwrap();
            println!(
                "Запись на дату {} уже существует, значение: {} мл",
                date, self.water_amount
            );
        } else {
            // Если записи нет, инициализируем количество воды нулем
            self.water_amount = 0;
            db.insert(
                date.as_bytes(),
                bincode::serialize(&self.water_amount).unwrap(),
            )?;
            db.flush()?;
            println!(
                "Создана новая запись для {} с начальным значением: 0 мл",
                date
            );
        }
        self.is_update = true;
        Ok(())
    }

    fn get_water_data(&self, db: &Db, date: String) -> Result<Option<u32>> {
        if let Some(stored_value) = db.get(date.as_bytes())? {
            let water_amount: u32 = bincode::deserialize(&stored_value).unwrap();
            Ok(Some(water_amount))
        } else {
            Ok(None)
        }
    }

    fn update_water_data(&self, db: &Db, date: String, new_amount: u32) -> sled::Result<()> {
        if (db.get(date.as_bytes())?).is_some() {
            db.insert(date.as_bytes(), bincode::serialize(&new_amount).unwrap())?;
            db.flush()?;
            println!("Запись для {} обновлена: {} мл", date, new_amount);
        } else {
            println!("Запись для {} не найдена.", date);
        }
        Ok(())
    }
}

pub fn combined_widget(
    ui: &mut Ui,
    food_widget: &mut FoodWidget,
    water_manager: &mut WaterManager,
) {
    let frame = Frame {
        fill: parse_color_from_ini("frame-background"),
        stroke: egui::Stroke::new(1.0, parse_color_from_ini("frame-border-color")),
        rounding: egui::Rounding::same(2.0),
        ..Default::default()
    };

    let container_rect = egui::Rect::from_min_size(Pos2::new(7.0, 410.0), Vec2::new(410.0, 140.0));
    ui.allocate_ui_at_rect(container_rect, |ui| {
        frame.show(ui, |ui| {
            let _ = food_widget.calorie_adjustment_widget(ui, frame);
            let _ = water_manager.water_widget(ui, frame);
        });
    });
}
