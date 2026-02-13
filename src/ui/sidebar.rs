use crate::ui::color_parser::parse_color_from_ini;
use crate::ui::health_widget::HealthWidget;
use crate::ui::settings::Settings;
use crate::ui::task_manager::TaskManager;
use crate::ui::weather_widget::WeatherWidget;
use crate::ui::aw_qt::SunburstWidget;
use crate::ui::notifications_listener::{NotificationsListener, Notification};

use egui::Context;
use std::sync::{Arc, Mutex};

#[derive(PartialEq)]
enum ViewMode {
    Widgets,
    Notifications,
}

pub(crate) struct SideBar {
    view_mode: ViewMode,
    task_manager: TaskManager,
    weather_widget: WeatherWidget,
    sunburst_widget: SunburstWidget,
    health_widget: HealthWidget,
    settings: Settings,
    notifications_listener: NotificationsListener,
    notifications: Arc<Mutex<Vec<Notification>>>,
}

impl SideBar {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_ui(&cc.egui_ctx);
        
        let notifications_listener = NotificationsListener::new();
        let notifications = notifications_listener.get_notifications();
        
        // Запускаем слушатель
        notifications_listener.start_listening(cc.egui_ctx.clone());
        
        Self {
            view_mode: ViewMode::Widgets,
            task_manager: TaskManager::default(),
            weather_widget: WeatherWidget::default(),
            sunburst_widget: SunburstWidget::new(),
            health_widget: HealthWidget::new(),
            settings: Settings::default(),
            notifications_listener,
            notifications,
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
            self.render_navigation_buttons(ui);
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

        let notif_count = self.notifications_listener.get_count();
        let button_text = if notif_count > 0 {
            format!("Notifications ({})", notif_count)
        } else {
            "Notifications".to_string()
        };

        if ui
            .add(
                egui::Button::new(button_text)
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
    // Виджеты теперь рисуются через обычный layout, без абсолютного позиционирования
    
    // Sunburst widget (Activity Watch visualization)
    //self.sunburst_widget.show_sunburst_widget(ui);
    //ui.add_space(10.0);
    
    // Weather widget
    //self.weather_widget.show_weather_widget(ui);
    //ui.add_space(10.0);
    
    // Tasks widget
    self.task_manager.show_tasks_widget(ui, ctx);
    ui.add_space(10.0);
    
    //Health widgets (food + water)
      self.health_widget.render(ui, ctx);
}

    fn render_notifications_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.heading("🔔 Notifications");
                
                let count = self.notifications_listener.get_count();
                ui.label(egui::RichText::new(format!("({})", count))
                    .size(12.0)
                    .color(egui::Color32::GRAY));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear All").clicked() {
                        self.notifications_listener.clear_all();
                    }
                });
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let notifications_clone = if let Ok(notifs) = self.notifications.lock() {
                        notifs.clone()
                    } else {
                        Vec::new()
                    };
                    
                    if notifications_clone.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("No notifications yet");
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Listening via dbus-monitor...")
                                .size(12.0)
                                .color(egui::Color32::GRAY));
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new("Try: notify-send 'Test' 'Message'")
                                .size(11.0)
                                .color(egui::Color32::DARK_GRAY)
                                .italics());
                        });
                    } else {
                        for notification in notifications_clone.iter().rev() {
                            self.render_notification_card(ui, notification);
                            ui.add_space(8.0);
                        }
                    }
                });
        });
    }

    fn render_notification_card(&mut self, ui: &mut egui::Ui, notification: &Notification) {
        let notification_id = notification.id;
        
        egui::Frame::none()
            .fill(parse_color_from_ini("button-color").linear_multiply(0.3))
            .rounding(8.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&notification.app_name)
                                    .strong()
                                    .size(13.0)
                            );
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(egui::RichText::new("✕")
                                    .size(14.0)
                                    .color(egui::Color32::from_rgb(200, 60, 60)))
                                    .on_hover_text("Remove notification")
                                    .clicked() 
                                {
                                    self.notifications_listener.remove_notification(notification_id);
                                }
                                
                                ui.add_space(5.0);
                                
                                ui.label(
                                    egui::RichText::new(&notification.timestamp)
                                        .size(11.0)
                                        .color(egui::Color32::GRAY)
                                );
                            });
                        });
                        
                        if !notification.summary.is_empty() {
                            ui.label(
                                egui::RichText::new(&notification.summary)
                                    .size(14.0)
                            );
                        }
                        
                        if !notification.body.is_empty() {
                            ui.label(
                                egui::RichText::new(&notification.body)
                                    .size(12.0)
                                    .color(egui::Color32::DARK_GRAY)
                            );
                        }
                    });
                });
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

        //if self.food_widget.calory_popup {
            //self.food_widget.calory_popup(ctx);
        //}
    }
}

impl eframe::App for SideBar {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(2.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            self.render_top_bar(ui, ctx);
            
            ui.add_space(10.0);
            ui.separator();
            
            match self.view_mode {
                ViewMode::Widgets => self.render_widgets_view(ui, ctx),
                ViewMode::Notifications => self.render_notifications_view(ui),
            }
        });

        self.render_popups(ctx);
    }
}