use bincode;
use chrono::prelude::*;
use egui::{Color32, Frame, Pos2, Rect, Vec2};
use sled::{Db, Result};

#[derive(Default)]
pub struct WaterManager {
    pub water_amount: u32,
    pub is_update: bool,
    pub is_first_call: bool,
}

impl WaterManager {
    pub fn water_widget(&mut self, ui: &mut egui::Ui) -> Result<()> {
        let date: String = self.get_date();

        // Открываем базу данных с обработкой ошибок
        let db: Db = self.open_db("~/.local/share/SideBarWaterDb")?;

        // Инициализируем запись на текущую дату и устанавливаем значение self.water_amount
        self.init_today_record(&db, date.clone())?;

        let frame = Frame {
            fill: Color32::from_rgb(255, 228, 225),
            stroke: egui::Stroke::new(1.0, Color32::from_rgb(253, 108, 158)),
            rounding: egui::Rounding::same(2.0),
            ..Default::default()
        };

        let container_rect =
            egui::Rect::from_min_size(Pos2::new(7.0, 410.0), Vec2::new(410.0, 140.0));
        ui.allocate_ui_at_rect(container_rect, |ui| {
            frame.show(ui, |ui| {
                if self.is_update || self.is_first_call {
                    match self.get_water_data(&db, date.clone()).unwrap() {
                        Some(amount) => {
                            let rect1 = egui::Rect::from_min_size(
                                Pos2::new(14.0, 410.0),
                                Vec2::new(90.0, 40.0),
                            );

                            ui.allocate_space(Vec2::new(438.0, 1.0));

                            ui.allocate_ui_at_rect(rect1, |ui| {
                                ui.allocate_space(Vec2::new(1.0, 3.0));
                                frame.show(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::Center),
                                        |ui| {
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
                                                            .fill(Color32::from_rgb(255, 228, 225))
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
                                                            .fill(Color32::from_rgb(255, 228, 225))
                                                            .min_size(Vec2 { x: 14.0, y: 10.0 }),
                                                    )
                                                    .clicked()
                                                {
                                                    self.water_amount =
                                                        self.water_amount.saturating_sub(400);
                                                    let _ = self.update_water_data(
                                                        &db,
                                                        date.clone(),
                                                        self.water_amount,
                                                    );
                                                    self.is_update = true;
                                                }
                                            });
                                            ui.allocate_space(Vec2::new(1.0, 3.0));
                                        },
                                    );
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
            });
        });
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
        if let Some(_) = db.get(date.as_bytes())? {
            db.insert(date.as_bytes(), bincode::serialize(&new_amount).unwrap())?;
            db.flush()?;
            println!("Запись для {} обновлена: {} мл", date, new_amount);
        } else {
            println!("Запись для {} не найдена.", date);
        }
        Ok(())
    }
}
