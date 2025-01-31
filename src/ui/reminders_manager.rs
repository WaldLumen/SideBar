use crate::ui::color_parser::parse_color_from_ini;
use egui::{Frame, Pos2, TextEdit, Vec2, Window};
use std::process::Command;

#[derive(Default)]
pub(crate) struct RemindersManager {
    pub reminder_text: String,
    pub reminder_time: String,
    pub is_new_reminder_opens: bool,
}

#[derive(Debug)]
struct Reminder {
    id: String,
    time: String,
    description: String,
}

impl RemindersManager {
    fn get_all_reminders(&mut self) -> Vec<Reminder> {
        // Get all reminder IDs
        let id_output = Command::new("sh")
            .arg("-c")
            .arg("atq | cut -f 1")
            .output()
            .expect("Failed to execute 'atq' command");

        let ids = String::from_utf8(id_output.stdout)
            .expect("Invalid UTF-8 sequence")
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        // Prepare to collect reminders
        let mut reminders = Vec::new();

        for id in &ids {
            // Get reminder time
            let time_output = Command::new("sh")
                .arg("-c")
                .arg(format!("atq {} | cut -d ' ' -f 4 | cut -d ':' -f 1,2", id))
                .output()
                .expect("Failed to execute 'atq' command");

            let time = String::from_utf8(time_output.stdout)
                .expect("Invalid UTF-8 sequence")
                .trim()
                .to_string();

            // Get reminder description
            let description_output = Command::new("sh")
                .arg("-c")
                .arg(format!("at -c {} | grep 'notify' | cut -d '\"' -f 2", id))
                .output()
                .expect("Failed to execute 'at' command");

            let description = String::from_utf8(description_output.stdout)
                .expect("Invalid UTF-8 sequence")
                .trim()
                .to_string();

            // Add reminder to the list
            reminders.push(Reminder {
                id: id.clone(),
                time,
                description,
            });
        }

        reminders
    }

    fn delete_reminder(&mut self, id: String) {
        Command::new("atrm")
            .arg(id)
            .output()
            .expect("Failed to execute 'atrm' command");
    }

    fn new_reminder(&mut self, text: String, time: String) {
        Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo 'notify-send \"{}\" -u normal -a Sidebar' | at {}",
                text, time
            ))
            .output()
            .expect("Failed to execute 'at' command");
    }

    pub fn create_reminder_popup(&mut self, ctx: &egui::Context) {
        Window::new("Edit Task")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(300.0, 500.0))
            .show(ctx, |ui| {
                ui.label("Reminder Text:");
                ui.add(
                    TextEdit::multiline(&mut self.reminder_text).min_size(Vec2::new(300.0, 100.0)),
                );

                ui.label("Reminder Time:");
                ui.add(
                    TextEdit::multiline(&mut self.reminder_time).min_size(Vec2::new(300.0, 50.0)),
                );

                ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                    if ui.button("Save").clicked() {
                        self.new_reminder(self.reminder_text.clone(), self.reminder_time.clone());
                        self.is_new_reminder_opens = false;
                    }

                    if ui.button("Close").clicked() {
                        self.is_new_reminder_opens = false;
                    }
                });
            });
    }

    pub fn reminder_manager(&mut self, ui: &mut egui::Ui) {
        let frame = Frame {
            fill: parse_color_from_ini("frame-background"),
            stroke: egui::Stroke::new(1.0, parse_color_from_ini("frame-border-color")),
            rounding: egui::Rounding::same(2.0),
            ..Default::default()
        };

        let container_rect =
            egui::Rect::from_min_size(Pos2::new(7.0, 548.0), Vec2::new(438.0, 170.0));

        ui.allocate_ui_at_rect(container_rect, |ui| {
            frame.show(ui, |ui| {
                ui.allocate_space(Vec2::new(438.0, 5.0));
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 50.0);
                    ui.label("Reminders: ");
                    ui.add_space(ui.available_width() - 20.0);
                    if ui
                        .add(
                            egui::Button::new("+")
                                .fill(parse_color_from_ini("button-color"))
                                .min_size(Vec2 { x: 2.0, y: 2.0 }),
                        )
                        .clicked()
                    {
                        self.is_new_reminder_opens = true;
                    }
                });

                let reminders: Vec<Reminder> = self.get_all_reminders();

                if !reminders.is_empty() {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .id_source("reminder_scroll_area")
                        .show(ui, |ui| {
                            for reminder in reminders {
                                if !reminder.description.is_empty() {
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::LEFT),
                                        |ui| {
                                            ui.allocate_space(egui::Vec2::new(2.0, 0.0));
                                            ui.label(format!(
                                                "  {} - {}",
                                                reminder.time, reminder.description
                                            ));

                                            if ui
                                                .add(
                                                    egui::Button::new("󰆴")
                                                        .fill(parse_color_from_ini("button-color"))
                                                        .min_size(Vec2 { x: 2.0, y: 2.0 })
                                                        .small(),
                                                )
                                                .clicked()
                                            {
                                                self.delete_reminder(reminder.id.clone());
                                            }
                                        },
                                    );
                                }
                            }
                        });
                } else {
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.label("There is nothing to do");
                        },
                    );
                }
            });

            ui.allocate_space(Vec2::new(438.0, 3.0));
        });
    }
}
