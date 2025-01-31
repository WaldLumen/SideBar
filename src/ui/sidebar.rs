use crate::ui::color_parser::parse_color_from_ini;
use crate::ui::health_widget::{combined_widget, FoodWidget, WaterManager};
use crate::ui::reminders_manager::RemindersManager;
use crate::ui::settings::Settings;
use crate::ui::task_manager::TaskManager;
use crate::ui::weather_widget::WeatherWidget;

use egui::{Context, Vec2};

pub(crate) struct SideBar {
    is_notifications: bool,
    task_manager: TaskManager,
    weather_widget: WeatherWidget,
    reminders_manager: RemindersManager,
    food_widget: FoodWidget,
    water_manager: WaterManager,
    settings: Settings,
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
            food_widget: FoodWidget::default(),
            water_manager: WaterManager::default(),
            settings: Settings::default(),
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
                    .add(
                        egui::Button::new("Notifications")
                            .min_size(Vec2 { x: 210.0, y: 20.0 })
                            .fill(parse_color_from_ini("button-color")),
                    )
                    .clicked()
                {
                    self.is_notifications = true;
                }
                if ui
                    .add(
                        egui::Button::new("Widgets")
                            .min_size(Vec2 { x: 210.0, y: 20.0 })
                            .fill(parse_color_from_ini("button-color")),
                    )
                    .clicked()
                {
                    self.is_notifications = false;
                }
            });

            if !self.is_notifications {
                self.weather_widget.show_weather_widget(ui);
                self.task_manager.show_tasks_widget(ui, ctx);
                combined_widget(ui, &mut self.food_widget, &mut self.water_manager);
                self.reminders_manager.reminder_manager(ui);
                self.settings.button_create(ui);

                if self.settings.popup_open {
                    self.settings.create_settings_window(ctx);
                }

                if self.task_manager.new_task_popup {
                    self.task_manager.new_task_popup(ctx);
                }

                if self.reminders_manager.is_new_reminder_opens {
                    self.reminders_manager.create_reminder_popup(ctx);
                }

                if self.task_manager.edit_task_popup {
                    self.task_manager.edit_task_popup(ctx);
                }

                if self.food_widget.calory_popup {
                    self.food_widget.calory_popup(ctx);
                }
            } else {
                ui.label("in progress");
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
