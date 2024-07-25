use crate::ui::widgets::todo_widget::{get_tasks, Task};
use egui::{Frame, Pos2, TextEdit, Vec2, Window, Color32};
use std::process::Command;


#[derive(Default)]
pub(crate) struct TaskManager {
    pub tasks: Vec<Task>,
    pub task_project: String,    
    pub task_description: String,
    
    pub current_task_id: Option<i32>,
    
    pub main_container_size: Vec2,

    pub new_task_popup: bool,
    pub edit_task_popup: bool,
    pub first_call: bool,
    pub is_update: bool,
}


impl TaskManager {
    pub fn modify_task(&mut self) {
	if let Some(task_id) = self.current_task_id {
            Command::new("task")
                .arg(format!("{}", task_id))
                .arg("modify")
                .arg("description:")
                .arg(format!("{}", self.task_description))
                .arg(format!("project: {}", self.task_project))
                .output()
                .expect("Failed to execute 'task' command"); 
        }
    }

    pub fn add_task(&mut self){
	Command::new("task")
            .arg("add")
            .arg(format!("{}", self.task_description))
	    .arg(format!("project: {}", self.task_project))
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
                ui.label("New task:");
                ui.add(TextEdit::multiline(&mut self.task_description).min_size(Vec2::new(300.0, 100.0)));
		ui.label("Project:");
		ui.add(TextEdit::multiline(&mut self.task_project).min_size(Vec2::new(300.0, 100.0)));
                if ui.button("Save").clicked() {
                    self.new_task_popup = false;
		    self.add_task();
		    self.is_update = true;
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
                ui.add(TextEdit::multiline(&mut self.task_description).min_size(Vec2::new(300.0, 100.0)));

		ui.label("Project:");
		ui.add(TextEdit::multiline(&mut self.task_project).min_size(Vec2::new(300.0, 100.0)));
		
                if ui.button("Save").clicked() {
                    self.edit_task_popup = false;
		    self.modify_task();
                }

                if ui.button("Close").clicked() {
                    self.edit_task_popup = false;
                }
            });
    }

    pub fn show_tasks_widget(&mut self, ui: &mut egui::Ui) {
    self.main_container_size = Vec2::new(436.0, 170.0);

    let frame = Frame {
        fill: Color32::from_rgb(255, 228, 225),
        stroke: egui::Stroke::new(1.0, Color32::from_rgb(253, 108, 158)),
        rounding: egui::Rounding::same(2.0),
        ..Default::default()
    };

    self.update_tasks();

    let container_rect = egui::Rect::from_min_size(Pos2::new(7.0, 71.0), self.main_container_size);

    let tasks = self.tasks.clone(); // Clone tasks to avoid borrow issues

    ui.allocate_ui_at_rect(container_rect, |ui| {
        frame.show(ui, |ui| {
            ui.label("                         Tasks:");

            self.show_add_task_button(ui);

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    self.show_tasks(ui, &tasks); // Pass the cloned tasks
                });
        });
    });
}

fn update_tasks(&mut self) {
    if !self.first_call {
        self.tasks = get_tasks();
        self.first_call = true;
    } else if self.is_update {
        self.tasks = get_tasks();
        self.is_update = false;
    }
}

fn show_add_task_button(&mut self, ui: &mut egui::Ui) {
    let rect = egui::Rect::from_min_size(Pos2::new(420.0, 72.0), Vec2::new(0.0, 0.0));
    ui.allocate_ui_at_rect(rect, |ui| {
        if ui
            .add(
                egui::Button::new("+")
                    .fill(Color32::from_rgb(255, 228, 225))
                    .min_size(Vec2 { x: 4.0, y: 4.0 }),
            )
            .clicked()
        {
	    self.task_description = " ".to_string();
	    self.task_project = " ".to_string();
            self.new_task_popup = true;
        }
    });
}

fn show_tasks(&mut self, ui: &mut egui::Ui, tasks: &Vec<Task>) {
    for task in tasks {
        let limit: usize = 42;
        let description = if task.description.chars().count() > limit {
            format!("{}...", &task.description.chars().take(limit).collect::<String>())
        } else {
            task.description.clone()
        };

        ui.horizontal(|ui| {
            ui.allocate_space(Vec2::new(2.0, 0.0));
            ui.label(format!(" {}", description));

            self.task_actions(ui, task.id);
        });
        ui.add_space(4.0); // Add space between tasks
    }
}
    
     

    fn task_actions(&mut self, ui: &mut egui::Ui, task_id: i32) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
            if ui
                .add(
                    egui::Button::new("󰆴")
                        .fill(Color32::from_rgb(255, 228, 225))
                        .min_size(Vec2 { x: 16.0, y: 16.0 }),
                )
                .clicked()
            {
                self.delete_task(task_id);
                self.is_update = true;
            }

            if ui
                .add(
                    egui::Button::new("󰄲")
                        .fill(Color32::from_rgb(255, 228, 225))
                        .min_size(Vec2 { x: 16.0, y: 16.0 }),
                )
                .clicked()
            {
                self.done_task(task_id);
                self.is_update = true;
            }

            if ui
                .add(
                    egui::Button::new("")
                        .fill(Color32::from_rgb(255, 228, 225))
                        .min_size(Vec2 { x: 16.0, y: 16.0 }),
                )
                .clicked()
            {
                self.current_task_id = Some(task_id);
                self.task_description = self.tasks.iter().find(|task| task.id == task_id).unwrap().description.clone();
		self.task_project = self.tasks.iter().find(|task| task.id == task_id).unwrap().project.clone();
		self.edit_task_popup = true;
                self.is_update = true;
            }
        });
    }

}
