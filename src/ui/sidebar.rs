use crate::ui::color_parser::parse_color_from_ini;
use crate::ui::health_widget::{combined_widget, FoodWidget, WaterManager};
use crate::ui::reminders_manager::RemindersManager;
use crate::ui::settings::Settings;
use crate::ui::task_manager::TaskManager;
use crate::ui::weather_widget::WeatherWidget;

use egui::Context;

#[derive(PartialEq)]
enum ViewMode {
    Widgets,
    Notifications,
}

pub(crate) struct SideBar {
    view_mode: ViewMode,
    task_manager: TaskManager,
    weather_widget: WeatherWidget,
    reminders_manager: RemindersManager,
    food_widget: FoodWidget,
    water_manager: WaterManager,
    settings: Settings,
}

impl SideBar {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_ui(&cc.egui_ctx);
        
        Self {
            view_mode: ViewMode::Widgets,
            task_manager: TaskManager::default(),
            weather_widget: WeatherWidget::default(),
            reminders_manager: RemindersManager::default(),
            food_widget: FoodWidget::default(),
            water_manager: WaterManager::default(),
            settings: Settings::default(),
        }
    }

    fn setup_ui(ctx: &egui::Context) {
        Self::setup_custom_fonts(ctx);
        Self::set_light_theme(ctx);
        Self::configure_style(ctx);
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
    }

    fn configure_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (egui::TextStyle::Heading, egui::FontId::new(16.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Body, egui::FontId::new(14.0, egui::FontFamily::Proportional)),
            (egui::TextStyle::Monospace, egui::FontId::new(18.0, egui::FontFamily::Monospace)),
            (egui::TextStyle::Button, egui::FontId::new(13.5, egui::FontFamily::Proportional)),
            (egui::TextStyle::Small, egui::FontId::new(12.0, egui::FontFamily::Proportional)),
        ]
        .into();

        ctx.set_style(style);
    }

    fn set_light_theme(ctx: &egui::Context) {
        ctx.set_visuals(egui::Visuals::light());
    }

    fn render_top_bar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            // Кнопки навигации
            self.render_navigation_buttons(ui);
            
            // Пушер - занимает все оставшееся пространство
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Кнопка настроек справа
                self.settings.button_create(ui, ctx);
            });
        });
    }

    fn render_navigation_buttons(&mut self, ui: &mut egui::Ui) {
        let button_spacing = 5.0;
        let button_width = 190.0;
        
        let notifications_color = if self.view_mode == ViewMode::Notifications {
            parse_color_from_ini("button-color").linear_multiply(1.2)
        } else {
            parse_color_from_ini("button-color")
        };

        let widgets_color = if self.view_mode == ViewMode::Widgets {
            parse_color_from_ini("button-color").linear_multiply(1.2)
        } else {
            parse_color_from_ini("button-color")
        };

        if ui
            .add(
                egui::Button::new("Widgets")
                    .min_size(egui::Vec2::new(button_width, 30.0))
                    .rounding(5.0)
                    .fill(widgets_color),
            )
            .clicked()
        {
            self.view_mode = ViewMode::Widgets;
        }

        ui.add_space(button_spacing);

        if ui
            .add(
                egui::Button::new("Notifications")
                    .min_size(egui::Vec2::new(button_width, 30.0))
                    .rounding(5.0)
                    .fill(notifications_color),
            )
            .clicked()
        {
            self.view_mode = ViewMode::Notifications;
        }
    }

    fn render_widgets_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_space(10.0);
        
        self.weather_widget.show_weather_widget(ui);
        ui.add_space(10.0);
        
        self.task_manager.show_tasks_widget(ui, ctx);
        ui.add_space(10.0);
        
        combined_widget(ui, &mut self.food_widget, &mut self.water_manager);
        ui.add_space(10.0);
        
        self.reminders_manager.reminder_manager(ui);
        ui.add_space(20.0);
    }

    fn render_notifications_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading("🔔 Notifications");
            ui.add_space(20.0);
            ui.label("Feature in progress...");
        });
    }

    fn render_popups(&mut self, ctx: &egui::Context) {
        if self.settings.popup_open {
            self.settings.create_settings_window(ctx);
        }

        if self.task_manager.new_task_popup {
            self.task_manager.new_task_popup(ctx);
        }

        if self.task_manager.edit_task_popup {
            self.task_manager.edit_task_popup(ctx);
        }

        if self.reminders_manager.is_new_reminder_opens {
            self.reminders_manager.create_reminder_popup(ctx);
        }

        if self.food_widget.calory_popup {
            self.food_widget.calory_popup(ctx);
        }
    }
}

impl eframe::App for SideBar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // Верхняя панель с навигацией и настройками
            self.render_top_bar(ui, ctx);
            
            ui.add_space(10.0);
            ui.separator();
            
            // Контент в зависимости от выбранной вкладки
            match self.view_mode {
                ViewMode::Widgets => self.render_widgets_view(ui, ctx),
                ViewMode::Notifications => self.render_notifications_view(ui),
            }
        });

        // Рендерим попапы после основного UI
        self.render_popups(ctx);
    }
}