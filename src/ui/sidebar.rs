use crate::ui::async_food_widget::FoodWidget;
use crate::ui::health_widget::WaterManager;
use crate::ui::reminders_manager::RemindersManager;
use crate::ui::task_manager::TaskManager;
use crate::ui::weather_widget::WeatherWidget;

use egui::{Context, Vec2};

pub(crate) struct SideBar {
    is_notifications: bool,
    task_manager: TaskManager,
    weather_widget: WeatherWidget,
    reminders_manager: RemindersManager,
    water_manager: WaterManager,
    food_widget: FoodWidget,
}

impl SideBar {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        set_light_theme(&cc.egui_ctx);
        Self {
            is_notifications: false,
            task_manager: TaskManager::default(),
            weather_widget: WeatherWidget::default(),
            reminders_manager: RemindersManager::default(),
            water_manager: WaterManager::default(),
            food_widget: FoodWidget::default(),
        }
    }
}

impl eframe::App for SideBar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let _show = egui::CentralPanel::default().show(ctx, |ui| {
            ui.allocate_space(Vec2::new(0.0, 30.0));
            ctx.set_pixels_per_point(2.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                if ui
                    .add(egui::Button::new("Notifications").min_size(Vec2 { x: 210.0, y: 20.0 }))
                    .clicked()
                {
                    self.is_notifications = true;
                }
                if ui
                    .add(egui::Button::new("Widgets").min_size(Vec2 { x: 210.0, y: 20.0 }))
                    .clicked()
                {
                    self.is_notifications = false;
                }
            });

            if !self.is_notifications {
                self.weather_widget.show_weather_widget(ui);
                self.task_manager.show_tasks_widget(ui, ctx);
                self.water_manager.water_widget(ui);
                self.food_widget.food_widget(ui);

                if self.task_manager.new_task_popup {
                    self.task_manager.new_task_popup(ctx);
                }

                if self.task_manager.edit_task_popup {
                    self.task_manager.edit_task_popup(ctx);
                }
            } else {
            }
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/JetBrainsMonoNerdFont-Medium.ttf"
        )),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("my_font".to_owned());
    ctx.set_fonts(fonts);

    let mut style = (*ctx.style()).clone();

    // Изменяем размер шрифтов для разных стилей
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(16.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(18.0, egui::FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(13.5, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(12.0, egui::FontFamily::Proportional),
    );

    // Применяем стиль
    ctx.set_style(style);
}

fn set_light_theme(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());
}
