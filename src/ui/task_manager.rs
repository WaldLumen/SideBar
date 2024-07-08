use crate::ui::widgets::todo_widget::get_tasks;
use egui::{Frame, Pos2, TextEdit, Vec2, Window};
use std::process::Command;

#[derive(Default)]
pub(crate) struct TaskManager {
    pub input_text: String,
    pub current_task_id: Option<i32>,
    pub new_task_popup: bool,
    pub edit_task_popup: bool,
    pub saved_text: String,
    pub main_container_size: Vec2,
}

#[derive(Default)]
pub(crate) struct Task {}

impl TaskManager {
    pub fn modify_task(&mut self) {
	if let Some(task_id) = self.current_task_id {
            Command::new("task")
                .arg(format!("{}", task_id))
                .arg("modify")
                .arg("description:")
                .arg(format!("{}", self.saved_text))
                .output()
                .expect("Failed to execute 'task' command"); 
        }
    }

    pub fn add_task(&mut self){
	Command::new("task")
            .arg("add")
            .arg(format!("{}", self.saved_text))
            .output()
            .expect("Failed to execute 'task' command");
    }

    pub fn delete_task(&mut self, task_id: i32){
	Command::new("task")
            .args(["rc.confirmation=no", "delete"])
            .arg(format!("{}", task_id))
            .output()
            .expect("Failed to execute 'task' command");
    }

    pub fn done_task(&mut self, task_id: i32){
	Command::new("task")
            .arg("done")
            .arg(format!("{}", task_id))
            .output()
            .expect("Failed to execute 'task' command");
    }

    
    pub fn new_task_popup(&mut self, ctx: &egui::Context) {
        Window::new("New Task")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
	    .fixed_size(Vec2::new(300.0, 500.0))
            .show(ctx, |ui| {
                ui.label("New task");
                ui.add(TextEdit::multiline(&mut self.input_text).min_size(Vec2::new(300.0, 100.0)));

                if ui.button("Save").clicked() {
                    self.saved_text = self.input_text.clone();
                    self.new_task_popup = false;
		    self.add_task();
                }

                if ui.button("Close").clicked() {
                    self.new_task_popup = false;
                }
            });
    }

    pub fn edit_task_popup(&mut self, ctx: &egui::Context) {
        Window::new("Edit Task")
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_size(Vec2::new(300.0, 500.0))
            .show(ctx, |ui| {
                ui.label("Edit task");
                ui.add(TextEdit::multiline(&mut self.input_text).min_size(Vec2::new(300.0, 100.0)));

                if ui.button("Save").clicked() {
                    self.saved_text = self.input_text.clone();
                    self.edit_task_popup = false;
		    self.modify_task();
                }

                if ui.button("Close").clicked() {
                    self.edit_task_popup = false;
                }
            });
    }


    pub fn show_tasks_widget(&mut self, ui: &mut egui::Ui) {
    self.main_container_size = Vec2::new(439.0, 170.0);

    let frame = Frame {       
        fill: egui::Color32::from_rgb(255, 228, 225),
        stroke: egui::Stroke::new(1.0, egui::Color32::from_rgb(253, 108, 158)),
        rounding: egui::Rounding::same(2.0),
        ..Default::default()
    };
    let vec = get_tasks();
    let container_rect = egui::Rect::from_min_size(Pos2::new(7.0, 71.0), self.main_container_size);

    ui.allocate_ui_at_rect(container_rect, |ui| {
        frame.show(ui, |ui| {
            ui.label("                         Tasks:");

            let rect = egui::Rect::from_min_size(Pos2::new(420.0, 72.0), Vec2::new(0.0, 0.0));
            ui.allocate_ui_at_rect(rect, |ui| {
                if ui
                    .add(
                        egui::Button::new("+")
                            .fill(egui::Color32::from_rgb(255, 228, 225))
                            .min_size(Vec2 { x: 4.0, y: 4.0 }),
                    )
                    .clicked()
                {
                    self.new_task_popup = true;
                    println!("new");
                }
            });

            // Создаем область с прокруткой
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let mut task_id = 0;
                    for item in vec {
                        for sub_item in item {
                            task_id += 1;
                            let limit: usize = 43;

                            let task = if sub_item.chars().count() > limit {
                                format!("{}...", &sub_item.chars().take(limit).collect::<String>())
                            } else {
                                sub_item.clone()
                            };

                            ui.horizontal(|ui| {
				ui.allocate_space(Vec2::new(3.0, 0.0));
                                ui.label(format!(" {}", task));

				ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                                    if ui
					.add(
                                            egui::Button::new("󰆴")
						.fill(egui::Color32::from_rgb(255, 228, 225))
						.min_size(Vec2 { x: 16.0, y: 16.0 }),
					)
					.clicked()
                                    {
					self.delete_task(task_id);
                                    }
				    
                                    if ui
					.add(
                                            egui::Button::new("󰄲")
						.fill(egui::Color32::from_rgb(255, 228, 225))
                                            .min_size(Vec2 { x: 16.0, y: 16.0 }),
					)
					.clicked()
                                    {
					self.done_task(task_id);
                                    }
				    
                                    if ui
					.add(
                                            egui::Button::new("")
						.fill(egui::Color32::from_rgb(255, 228, 225))
						.min_size(Vec2 { x: 16.0, y: 16.0 }),
					)
					.clicked()
                                    {
					self.current_task_id = Some(task_id);
					self.edit_task_popup = true;
					self.input_text = sub_item.clone(); // Предзаполнение текущим текстом задачи
                                    }
				});
			    });
				ui.add_space(4.0); // Добавляем пространство между задачами
                        }
                    }
                });
        });
    });
}
}
