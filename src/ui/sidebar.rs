use egui::{Vec2, Context};
use crate::ui::task_manager::TaskManager;
use crate::ui::weather_widget::WeatherWidget;

pub(crate) struct SideBar {
    is_widget: bool,
    task_manager: TaskManager,
    weather_widget: WeatherWidget,
}

impl SideBar {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        set_light_theme(&cc.egui_ctx);
        Self {
            is_widget: false,
            task_manager: TaskManager::default(),
            weather_widget: WeatherWidget::default(),
        }
    }
}

impl eframe::App for SideBar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let _show = egui::CentralPanel::default().show(ctx, |ui| {

	    ui.allocate_space(Vec2::new(0.0, 30.0));
            ctx.set_pixels_per_point(2.0);
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                if ui.add(egui::Button::new("Widgets").min_size(Vec2 { x: 210.0, y: 20.0 })).clicked() {
                    self.is_widget = true;
                }
                if ui.add(egui::Button::new("Notifications").min_size(Vec2 { x: 210.0, y: 20.0 })).clicked() {
                    self.is_widget = false;
                }
            });

            if ! self.is_widget {


		self.weather_widget.show_weather_widget(ui);
		
                self.task_manager.show_tasks_widget(ui);

                if self.task_manager.new_task_popup {
                    self.task_manager.new_task_popup(ctx);
                }

                if self.task_manager.edit_task_popup {
                    self.task_manager.edit_task_popup(ctx);
                }

                // Используем метод для отображения погоды из WeatherWidget
                //self.weather_widget.show_weather_widget(ui);
            }
        });
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMonoNerdFont-Medium.ttf")),
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

fn set_light_theme(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::light());
}
