use crate::ui::color_parser::parse_color_from_ini;
use crate::ui::settings::{get_daily_water_goal, get_water_increment, get_daily_calorie_goal};
use bincode;
use calory_fetch::{fetch_calory_of_certain_food, fetch_data};
use chrono::{DateTime, Local};
use egui::{Frame, TextEdit, Ui, Vec2, Window};
use sled::{Db, Result};
use std::env;
use std::path::PathBuf;
use tokio::runtime::Runtime;


const GRAMS_PER_100G: i32 = 100;
const NARROW_WINDOW_THRESHOLD: f32 = 600.0; // Порог для переключения на вертикальную верстку


#[derive(Default)]
pub struct FoodWidget {
    query: String,
    dish_name: String,
    results: Vec<(String, String)>,
    dish_calory: i32,
    selected_food: Option<(String, String)>,
    db: Option<Db>,
    calory: i32,
    pub calory_popup: bool,
    food_amount: String,
    runtime: Option<Runtime>,
}

impl FoodWidget {
    pub fn new() -> Self {
        Self {
            runtime: Some(Runtime::new().unwrap()),
            ..Default::default()
        }
    }

    fn get_db_path() -> PathBuf {
        let home = env::var("HOME").expect("Could not determine HOME directory");
        PathBuf::from(home).join(".local/share/SideBarFoodDb")
    }

    fn ensure_db(&mut self) -> Result<()> {
        if self.db.is_none() {
            let db = sled::open(Self::get_db_path())?;
            self.db = Some(db);
            self.init_today_record()?;
        }
        Ok(())
    }

    fn get_date() -> String {
        Local::now().format("%d.%m").to_string()
    }

    fn init_today_record(&mut self) -> Result<()> {
        if let Some(db) = &self.db {
            let date = Self::get_date();
            
            match db.get(date.as_bytes())? {
                Some(stored_value) => {
                    self.calory = bincode::deserialize(&stored_value)
                        .unwrap_or(0);
                }
                None => {
                    self.calory = 0;
                    db.insert(date.as_bytes(), bincode::serialize(&self.calory).unwrap())?;
                    db.flush()?;
                }
            }
        }
        Ok(())
    }

    fn update_calories(&mut self, additional: i32) -> Result<()> {
        if let Some(db) = &self.db {
            let date = Self::get_date();
            self.calory += additional;
            db.insert(date.as_bytes(), bincode::serialize(&self.calory).unwrap())?;
            db.flush()?;
        }
        Ok(())
    }

    pub fn render_popup(&mut self, ctx: &egui::Context) {
        if !self.calory_popup {
            return;
        }

        let bg_color = parse_color_from_ini("background-color");
        let text_color = parse_color_from_ini("text-color");
        let button_color = parse_color_from_ini("button-color");

        let screen_size = ctx.screen_rect().size();
        let popup_width = (screen_size.x * 0.9).min(350.0).max(280.0);
        let popup_height = 300.0;

        Window::new("Add Food")
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(popup_width, popup_height))
            .frame(Frame::window(&ctx.style()).fill(bg_color))
            .show(ctx, |ui| {
                ui.style_mut().visuals.override_text_color = Some(text_color);

                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    
                    ui.heading(&self.dish_name);
                    ui.add_space(5.0);
                    
                    ui.label(format!("{} kcal per 100g", self.dish_calory));
                    ui.add_space(15.0);
                    
                    ui.label("Amount (grams):");
                    ui.add(
                        TextEdit::singleline(&mut self.food_amount)
                            .hint_text("Enter amount...")
                            .desired_width(ui.available_width() * 0.8)
                    );
                    ui.add_space(10.0);

                    if let Ok(amount) = self.food_amount.parse::<i32>() {
                        let total_calories = (amount * self.dish_calory) / GRAMS_PER_100G;
                        
                        ui.label(format!("Total: {} kcal", total_calories));
                        ui.add_space(15.0);

                        let button_width = (ui.available_width() * 0.45).min(120.0);
                        
                        ui.horizontal(|ui| {
                            if ui.add(
                                egui::Button::new("✓ Add")
                                    .min_size(Vec2::new(button_width, 35.0))
                                    .fill(button_color)
                            ).clicked() {
                                let _ = self.update_calories(total_calories);
                                self.calory_popup = false;
                                self.food_amount.clear();
                            }

                            if ui.add(
                                egui::Button::new("✖ Cancel")
                                    .min_size(Vec2::new(button_width, 35.0))
                                    .fill(button_color.linear_multiply(0.7))
                            ).clicked() {
                                self.calory_popup = false;
                                self.food_amount.clear();
                            }
                        });
                    } else {
                        ui.label("Please enter a valid number");
                    }
                });
            });
    }

    fn render_search(&mut self, ui: &mut Ui) {
        let button_color = parse_color_from_ini("button-color");
        let available_width = ui.available_width();
        let is_narrow = available_width < 300.0;

        if is_narrow {
            ui.vertical(|ui| {
                ui.add(
                    TextEdit::singleline(&mut self.query)
                        .hint_text("Search food...")
                        .desired_width(ui.available_width())
                );
                
                ui.add_space(5.0);

                if ui.add(
                    egui::Button::new("🔍 Search")
                        .min_size(Vec2::new(ui.available_width(), 25.0))
                        .fill(button_color)
                ).clicked() {
                    if let Some(rt) = &self.runtime {
                        let results = rt.block_on(fetch_data(&self.query));
                        self.results = results
                            .into_iter()
                            .map(|item| (item.value, item.url))
                            .collect();
                    }
                }
            });
        } else {
            ui.horizontal(|ui| {
                let search_width = (available_width * 0.65).max(150.0);
                
                ui.add(
                    TextEdit::singleline(&mut self.query)
                        .hint_text("Search food...")
                        .desired_width(search_width)
                );

                if ui.add(
                    egui::Button::new("🔍 Search")
                        .min_size(Vec2::new(80.0, 25.0))
                        .fill(button_color)
                ).clicked() {
                    if let Some(rt) = &self.runtime {
                        let results = rt.block_on(fetch_data(&self.query));
                        self.results = results
                            .into_iter()
                            .map(|item| (item.value, item.url))
                            .collect();
                    }
                }
            });
        }
    }

    fn render_results(&mut self, ui: &mut Ui) {
        let button_color = parse_color_from_ini("button-color");
        let available_width = ui.available_width();
        let max_height = if available_width < 300.0 { 120.0 } else { 80.0 };

        egui::ScrollArea::vertical()
            .id_source("food_search_results")
            .max_height(max_height)
            .show(ui, |ui| {
                for (name, url) in self.results.clone() {
                    if ui.add(
                        egui::Button::new(&name)
                            .min_size(Vec2::new(ui.available_width(), 30.0))
                            .fill(button_color)
                            .wrap(true) // Перенос текста для узких окон
                    ).clicked() {
                        if let Some(rt) = &self.runtime {
                            let calory_data = rt.block_on(fetch_calory_of_certain_food(url));
                            
                            self.dish_name = calory_data.get(1).cloned().unwrap_or_default();
                            self.dish_calory = calory_data
                                .get(0)
                                .and_then(|s| s.chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .collect::<String>()
                                    .parse()
                                    .ok())
                                .unwrap_or(0);
                            
                            self.food_amount = String::from("100");
                            self.calory_popup = true;
                        }
                    }
                }
            });
    }

    pub fn render(&mut self, ui: &mut Ui) -> Result<()> {
        let _ = self.ensure_db();

        ui.vertical(|ui| {
            ui.heading("🍽 Food Tracker");
            ui.add_space(5.0);

            let available_width = ui.available_width();
            let is_very_narrow = available_width < 200.0;

            if is_very_narrow {
                ui.label(format!("Date: {}", Self::get_date()));
                ui.label(format!("Cal: {} / {}", self.calory, get_daily_calorie_goal()));
            } else {
                ui.horizontal(|ui| {
                    ui.label("Date:");
                    ui.label(Self::get_date());
                });

                ui.horizontal(|ui| {
                    ui.label("Calories:");
                    ui.label(format!("{} / {} kcal", self.calory, get_daily_calorie_goal()));
                });
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Search
            self.render_search(ui);
            ui.add_space(5.0);

            // Results
            if !self.results.is_empty() {
                self.render_results(ui);
            }
        });

        Ok(())
    }
}


#[derive(Default)]
pub struct WaterWidget {
    water_amount: u32,
    db: Option<Db>,
    initialized: bool,
}

impl WaterWidget {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_db_path() -> PathBuf {
        let home = env::var("HOME").expect("Could not determine HOME directory");
        PathBuf::from(home).join(".local/share/SideBarWaterDb")
    }

    fn get_date() -> String {
        Local::now().format("%d.%m").to_string()
    }

    fn ensure_db(&mut self) -> Result<()> {
        if self.db.is_none() {
            let db = sled::open(Self::get_db_path())?;
            self.db = Some(db);
        }

        if !self.initialized {
            self.init_today_record()?;
            self.initialized = true;
        }

        Ok(())
    }

    fn init_today_record(&mut self) -> Result<()> {
        if let Some(db) = &self.db {
            let date = Self::get_date();

            match db.get(date.as_bytes())? {
                Some(stored_value) => {
                    self.water_amount = bincode::deserialize(&stored_value)
                        .unwrap_or(0);
                }
                None => {
                    self.water_amount = 0;
                    db.insert(date.as_bytes(), bincode::serialize(&self.water_amount).unwrap())?;
                    db.flush()?;
                }
            }
        }
        Ok(())
    }

    fn update_water(&mut self, new_amount: u32) -> Result<()> {
        if let Some(db) = &self.db {
            let date = Self::get_date();
            self.water_amount = new_amount;
            db.insert(date.as_bytes(), bincode::serialize(&self.water_amount).unwrap())?;
            db.flush()?;
        }
        Ok(())
    }

    pub fn render(&mut self, ui: &mut Ui) -> Result<()> {
        let _ = self.ensure_db();
        let button_color = parse_color_from_ini("button-color");
        let available_width = ui.available_width();
        let is_very_narrow = available_width < 200.0;
        

        let daily_water_goal = get_daily_water_goal();
        let water_increment = get_water_increment();

        ui.vertical(|ui| {
            ui.heading("💧 Water Tracker");
            ui.add_space(5.0);

            if is_very_narrow {
                ui.label(format!("Date: {}", Self::get_date()));
                ui.label(format!("{} / {} ml", self.water_amount, daily_water_goal));
            } else {
                ui.horizontal(|ui| {
                    ui.label("Date:");
                    ui.label(Self::get_date());
                });

                ui.horizontal(|ui| {
                    ui.label("Amount:");
                    ui.label(format!("{} / {} ml", self.water_amount, daily_water_goal));
                });
            }

            ui.add_space(10.0);


            let progress = (self.water_amount as f32) / (daily_water_goal as f32);
            ui.add(egui::ProgressBar::new(progress).show_percentage());

            ui.add_space(10.0);

            let is_narrow = available_width < 250.0;
            
            if is_narrow {
                if ui.add(
                    egui::Button::new(format!("+ {} ml", water_increment))
                        .min_size(Vec2::new(ui.available_width(), 30.0))
                        .fill(button_color)
                ).clicked() {
                    let _ = self.update_water(self.water_amount + water_increment);
                }

                ui.add_space(5.0);

                if ui.add(
                    egui::Button::new(format!("- {} ml", water_increment))
                        .min_size(Vec2::new(ui.available_width(), 30.0))
                        .fill(button_color.linear_multiply(0.7))
                ).clicked() {
                    let _ = self.update_water(self.water_amount.saturating_sub(water_increment));
                }
            } else {
                ui.horizontal(|ui| {
                    let button_width = (ui.available_width() * 0.48).min(100.0);
                    
                    if ui.add(
                        egui::Button::new(format!("+ {} ml", water_increment))
                            .min_size(Vec2::new(button_width, 30.0))
                            .fill(button_color)
                    ).clicked() {
                        let _ = self.update_water(self.water_amount + water_increment);
                    }

                    if ui.add(
                        egui::Button::new(format!("- {} ml", water_increment))
                            .min_size(Vec2::new(button_width, 30.0))
                            .fill(button_color.linear_multiply(0.7))
                    ).clicked() {
                        let _ = self.update_water(self.water_amount.saturating_sub(water_increment));
                    }
                });
            }
        });

        Ok(())
    }
}

pub struct HealthWidget {
    pub food_widget: FoodWidget,
    pub water_widget: WaterWidget,
}

impl Default for HealthWidget {
    fn default() -> Self {
        Self {
            food_widget: FoodWidget::new(),
            water_widget: WaterWidget::new(),
        }
    }
}

impl HealthWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        self.food_widget.render_popup(ctx);

        let frame = Frame {
            fill: parse_color_from_ini("frame-background"),
            stroke: egui::Stroke::new(1.0, parse_color_from_ini("frame-border-color")),
            rounding: egui::Rounding::same(8.0),
            inner_margin: egui::Margin::same(15.0),
            ..Default::default()
        };

        frame.show(ui, |ui| {
            let available_width = ui.available_width();
            if available_width >= NARROW_WINDOW_THRESHOLD {
                ui.columns(2, |columns| {
                    let _ = self.food_widget.render(&mut columns[0]);
                    // Right column - Water tracker
                    let _ = self.water_widget.render(&mut columns[1]);
                });
            } else {
                
                let _ = self.food_widget.render(ui);
                
                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);
                
                let _ = self.water_widget.render(ui);
            }
        });
    }
}


pub fn combined_widget(
    ui: &mut Ui,
    ctx: &egui::Context,
    health_widget: &mut HealthWidget,
) {
    health_widget.render(ui, ctx);
}