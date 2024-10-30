use bincode;
use calory_fetch::{fetch_calory_of_certain_food, fetch_data, CaloryFetch};
use chrono::prelude::*;
use egui::{Color32, Frame, Pos2, Rect, ScrollArea, Ui, Vec2};
use sled::{Db, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::task;
use tokio::time::sleep;

// Подключаем функции и структуру из вашей библиотеки
pub struct FoodWidget {
    pub calory: i32,
    pub is_first_call: bool,
    pub search_query: String,
    pub last_search_query: String,
    pub filtered_results: Vec<CaloryFetch>,
    pub cached_results: HashMap<String, Vec<CaloryFetch>>,
    pub db: Option<Db>,
    pub last_search_time: Option<Instant>,
}

impl Default for FoodWidget {
    fn default() -> Self {
        Self {
            calory: 0,
            is_first_call: true,
            search_query: String::new(),
            last_search_query: String::new(),
            filtered_results: Vec::new(),
            cached_results: HashMap::new(),
            db: None,
            last_search_time: None,
        }
    }
}

impl FoodWidget {
    pub fn food_widget(&mut self, ui: &mut Ui) -> Result<()> {
        // Инициализируем базу данных и обрабатываем результаты поиска
        if self.is_first_call {
            self.db = Some(self.open_db("$HOME/.local/share/SideBarFoodDb")?);
            self.is_first_call = false;
            task::spawn(async {
                self.init_today_record().await;
            });
        }

        // Оформляем UI
        let frame = Frame {
            fill: Color32::from_rgb(255, 228, 225),
            stroke: egui::Stroke::new(1.0, Color32::from_rgb(253, 108, 158)),
            rounding: egui::Rounding::same(2.0),
            ..Default::default()
        };

        let container_rect = Rect::from_min_size(Pos2::new(7.0, 450.0), Vec2::new(410.0, 100.0));
        ui.allocate_ui_at_rect(container_rect, |ui| {
            frame.show(ui, |ui| {
                ui.allocate_space(Vec2::new(438.0, 1.0));
                self.search_ui(ui);
                self.display_results(ui);
            });
        });

        Ok(())
    }

    fn search_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let _response = ui.text_edit_singleline(&mut self.search_query);

            // Проверка на нажатие кнопки "Поиск"
            if ui.button("Search").clicked() {
                if self.should_perform_search() {
                    let query = self.search_query.clone();
                    let cache = self.cached_results.clone();
                    let db = self.db.clone();
                    task::spawn(async move {
                        self.perform_search(query, cache, db).await;
                    });
                }
            }
        });
    }

    fn should_perform_search(&mut self) -> bool {
        let now = Instant::now();
        if let Some(last_time) = self.last_search_time {
            if now.duration_since(last_time) < Duration::from_millis(500) {
                return false;
            }
        }
        self.last_search_time = Some(now);
        true
    }

    async fn perform_search(
        &mut self,
        query: String,
        cache: HashMap<String, Vec<CaloryFetch>>,
        db: Option<Db>,
    ) {
        if query == self.last_search_query {
            return;
        }
        self.last_search_query = query.clone();

        if let Some(cached_data) = cache.get(&query) {
            self.filtered_results = cached_data.clone();
            return;
        }

        if !query.is_empty() {
            if let Ok(results) = fetch_data(&query).await {
                self.filtered_results = results.clone();
                self.cached_results.insert(query.clone(), results);
            } else {
                self.filtered_results.clear();
            }
        } else {
            self.filtered_results.clear();
        }
    }

    fn display_results(&self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            if !self.filtered_results.is_empty() {
                for result in &self.filtered_results {
                    let url = result.url.clone();
                    task::spawn(async move {
                        if let Ok(food_info) = fetch_calory_of_certain_food(url).await {
                            let display_text =
                                format!("{}: {} калорий", food_info[1], food_info[0]);
                            ui.label(display_text);
                        }
                    });
                }
            } else if !self.search_query.is_empty() {
                ui.label("No results found");
            }
        });
    }

    fn open_db(&self, path: &str) -> Result<Db> {
        sled::open(path)
    }

    async fn init_today_record(&mut self) {
        if let Some(db) = &self.db {
            let date = self.get_date();
            if let Some(stored_value) = db.get(date.as_bytes()).await.unwrap() {
                self.calory = bincode::deserialize(&stored_value).unwrap();
                println!(
                    "Запись на дату {} уже существует, значение: {} калорий",
                    date, self.calory
                );
            } else {
                self.calory = 0;
                db.insert(date.as_bytes(), bincode::serialize(&self.calory).unwrap())
                    .await
                    .unwrap();
                db.flush().await.unwrap();
                println!(
                    "Создана новая запись для {} с начальным значением: 0 калорий",
                    date
                );
            }
        }
    }

    fn get_date(&self) -> String {
        let local: DateTime<Local> = Local::now();
        local.format("%d.%m").to_string()
    }
}
