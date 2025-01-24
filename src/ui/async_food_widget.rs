use bincode;
use calory_fetch::{fetch_calory_of_certain_food, fetch_data};
use chrono::{DateTime, Local};
use egui::{Color32, Frame, Pos2, Rect, Ui, Vec2};
use sled::{Db, Result};
use tokio::runtime::Runtime;

#[derive(Default)]
pub struct FoodWidget {
    query: String,
    results: Vec<(String, String)>, // Хранит список (название и URL) для каждого результата
    selected_food: Option<(String, String)>, // Хранит выбранное название и калорийность блюда
    db: Option<Db>,                 // Опциональная база данных
    calory: i32,                    // Калорийность для текущей даты
    calory_query: String,
}

impl FoodWidget {
    pub fn food_widget(&mut self, ui: &mut Ui) -> Result<()> {
        let rt = Runtime::new().unwrap();

        // Открываем базу данных при первом вызове
        if self.db.is_none() {
            self.db = Some(self.open_db("$HOME/.local/share/SideBarFoodDb")?);
            self.init_today_record();
        }

        let frame = Frame {
            fill: Color32::from_rgb(255, 228, 225),
            stroke: egui::Stroke::new(1.0, Color32::from_rgb(253, 108, 158)),
            rounding: egui::Rounding::same(2.0),
            inner_margin: egui::Margin::same(8.0),
            ..Default::default()
        };

        let container_rect = Rect::from_min_size(Pos2::new(7.0, 550.0), Vec2::new(480.0, 200.0));
        ui.allocate_ui_at_rect(container_rect, |ui| {
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(314.0, 30.0),
                        egui::TextEdit::singleline(&mut self.query),
                    );

                    if ui
                        .add_sized(
                            Vec2::new(100.0, 30.0),
                            egui::Button::new("Поиск").fill(Color32::from_rgb(255, 228, 225)),
                        )
                        .clicked()
                    {
                        let result = rt.block_on(fetch_data(&self.query));
                        self.results = result
                            .into_iter()
                            .map(|item| (item.value.clone(), item.url.clone()))
                            .collect();
                    }
                });

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .max_width(420.0)
                    .id_source("food_results_scroll_area")
                    .show(ui, |ui| {
                        for (name, url) in self.results.clone() {
                            if ui
                                .add_sized(
                                    Vec2::new(ui.available_width(), 30.0),
                                    egui::Button::new(name).fill(Color32::from_rgb(255, 228, 225)),
                                )
                                .clicked()
                            {
                                let calory = rt.block_on(fetch_calory_of_certain_food(url.clone()));

                                // Сохраняем выбранное блюдо
                                self.selected_food = Some((calory[1].clone(), calory[0].clone()));

                                // Получаем калорийность как целое число
                                let additional_calory: i32 = calory[0].parse().unwrap_or(0);

                                // Обновляем данные калорийности вне замыкания
                                self.update_today_calory(additional_calory);
                            }
                        }
                    });

                if let Some((name, calory)) = &self.selected_food {
                    ui.allocate_space(Vec2::new(0.0, 2.0));
                    ui.label(format!("{}: {} (100гр)", name, calory));
                }
            });
        });

        self.calorie_adjustment_widget(ui);

        Ok(())
    }

    fn open_db(&self, path: &str) -> Result<Db> {
        sled::open(path)
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

    fn calorie_adjustment_widget(&mut self, ui: &mut Ui) {
        let frame = Frame {
            fill: Color32::from_rgb(255, 228, 225),
            stroke: egui::Stroke::new(1.0, Color32::from_rgb(253, 108, 158)),
            rounding: egui::Rounding::same(2.0),
            inner_margin: egui::Margin::same(8.0),
            ..Default::default()
        };

        let container_rect = Rect::from_min_size(Pos2::new(7.0, 730.0), Vec2::new(480.0, 200.0));
        ui.allocate_ui_at_rect(container_rect, |ui| {
            frame.show(ui, |ui| {
                ui.allocate_space(Vec2::new(421.0, 1.0));
                ui.label(format!("Текущие калории: {}", self.calory));

                // Поле для ввода количества калорий для добавления или вычитания
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(50.0, 25.0),
                        egui::TextEdit::singleline(&mut self.calory_query),
                    );

                    if let Ok(calory_value) = self.calory_query.parse::<i32>() {
                        if ui.button("-").clicked() {
                            self.update_today_calory(-calory_value); // Уменьшение калорийности
                        }
                        if ui.button("+").clicked() {
                            self.update_today_calory(calory_value); // Увеличение калорийности
                        }
                    } else {
                        ui.label("Введите числовое значение калорий.");
                    }
                });
            });
        });
    }

    fn get_date(&self) -> String {
        let local: DateTime<Local> = Local::now();
        local.format("%d.%m").to_string()
    }
}
