use bincode;
use calory_fetch::{fetch_calory_of_certain_food, fetch_data};
use chrono::{DateTime, Local};
use egui::{Color32, Frame, Pos2, Ui, Vec2};
use sled::{Db, Result};
use tokio::runtime::Runtime;

#[derive(Default)]
pub struct FoodWidget {
    query: String,
    results: Vec<(String, String)>, // Хранит список (название и URL) для каждого результата
    selected_food: Option<(String, String)>, // Хранит выбранное название и калорийность блюда
    db: Option<Db>,                 // Опциональная база данных
    calory: i32,                    // Калорийность для текущей даты
}

impl FoodWidget {
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

    fn calorie_adjustment_widget(&mut self, ui: &mut Ui, frame: Frame) -> Result<()> {
        let rt = Runtime::new().unwrap();
        let date = self.get_date();
        if self.db.is_none() {
            self.db = Some(self.open_db("$HOME/.local/share/SideBarFoodDb")?);
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

                            ui.add_sized(
                                Vec2::new(135.0, 25.0),
                                egui::TextEdit::singleline(&mut self.query),
                            );

                            if ui
                                .add_sized(Vec2::new(80.0, 25.0), egui::Button::new("Поиск"))
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
                                            egui::Button::new(name),
                                        )
                                        .clicked()
                                    {
                                        let calory =
                                            rt.block_on(fetch_calory_of_certain_food(url.clone()));

                                        // Сохраняем выбранное блюдо
                                        self.selected_food =
                                            Some((calory[1].clone(), calory[0].clone()));

                                        // Получаем калорийность как целое число
                                        let additional_calory: i32 = calory[0].parse().unwrap_or(0);

                                        // Обновляем данные калорийности вне замыкания
                                        self.update_today_calory(additional_calory);
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
}

impl WaterManager {
    pub fn water_widget(&mut self, ui: &mut egui::Ui, frame: Frame) -> Result<()> {
        let date: String = self.get_date();

        // Открываем базу данных с обработкой ошибок
        let db: Db = self.open_db("~/.local/share/SideBarWaterDb")?;

        // Инициализируем запись на текущую дату и устанавливаем значение self.water_amount
        self.init_today_record(&db, date.clone())?;

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
                                                .min_size(Vec2 { x: 14.0, y: 10.0 }),
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
