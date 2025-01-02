use crate::ui::widgets::todo_widget::{get_tasks, Task};
use egui::{Color32, Frame, Key, Pos2, Rect, TextEdit, Vec2, Window};
use std::process::Command;

pub(crate) struct TaskManager {
    pub tasks: Vec<Task>,
    pub task_project: String,
    pub task_description: String,
    pub project_category: String,
    pub project_names: Vec<String>,
    pub current_task_id: Option<i32>,
    pub main_container_size: Vec2,
    pub new_task_popup: bool,
    pub edit_task_popup: bool,
    pub first_call: bool,
    pub is_update: bool,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: Vec::default(),
            task_project: String::default(),
            task_description: String::default(),
            project_category: "All".to_string(),
            current_task_id: Some(i32::default()),
            main_container_size: Vec2::default(),
            new_task_popup: false,
            edit_task_popup: false,
            first_call: true,
            is_update: false,
            project_names: Vec::default(),
        }
    }
}

impl TaskManager {
    pub fn get_project_names(&mut self) {
        if self.project_names.is_empty() {
            let output = Command::new("sh")
                .arg("-c")
                .arg("task projects | awk 'NR>3 && !/projects/ {print $1}'")
                .output()
                .expect("Failed to execute 'task' command");

            let output_str = String::from_utf8_lossy(&output.stdout);

            self.project_names = output_str
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s.chars().all(|c| !c.is_numeric()))
                .collect();
        }
    }

    pub fn modify_task(&mut self) {
        if let Some(task_id) = self.current_task_id {
            Command::new("task")
                .arg(format!("{}", task_id))
                .arg("modify")
                .arg("description:")
                .arg(self.task_description.clone())
                .arg(format!("project:{}", self.task_project))
                .output()
                .expect("Failed to execute 'task' command");
        }
    }

    pub fn add_task(&mut self) {
        Command::new("task")
            .arg("add")
            .arg(self.task_description.clone())
            .arg(format!("project:{}", self.task_project))
            .output()
            .expect("Failed to execute 'task' command");
    }

    pub fn delete_task(&mut self, task_id: i32) {
        Command::new("task")
            .args(["rc.confirmation=no", "delete"])
            .arg(format!("{}", task_id))
            .output()
            .expect("Failed to execute 'task' command");
    }

    pub fn done_task(&mut self, task_id: i32) {
        Command::new("task")
            .arg("done")
            .arg(format!("{}", task_id))
            .output()
            .expect("Failed to execute 'task' command");
    }

    pub fn new_task_popup(&mut self, ctx: &egui::Context) {
        if self.new_task_popup {
            Window::new("New Task")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .fixed_size(Vec2::new(300.0, 500.0))
                .show(ctx, |ui| {
                    ui.label("New task:");
                    ui.add(
                        TextEdit::multiline(&mut self.task_description)
                            .min_size(Vec2::new(300.0, 100.0)),
                    );
                    ui.label("Project:");
                    ui.add(
                        TextEdit::multiline(&mut self.task_project)
                            .min_size(Vec2::new(300.0, 100.0)),
                    );
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
    }

    pub fn edit_task_popup(&mut self, ctx: &egui::Context) {
        if self.edit_task_popup {
            Window::new("Edit Task")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .fixed_size(Vec2::new(300.0, 500.0))
                .show(ctx, |ui| {
                    ui.label("Edit task");
                    ui.add(
                        TextEdit::multiline(&mut self.task_description)
                            .min_size(Vec2::new(300.0, 100.0)),
                    );

                    ui.label("Project:");
                    ui.add(
                        TextEdit::multiline(&mut self.task_project)
                            .min_size(Vec2::new(300.0, 100.0)),
                    );

                    if ui.button("Save").clicked() {
                        self.edit_task_popup = false;
                        self.modify_task();
                    }

                    if ui.button("Close").clicked() {
                        self.edit_task_popup = false;
                    }
                });
        }
    }

    pub fn show_tasks_widget(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());

        if input.key_pressed(Key::N) && input.modifiers.ctrl {
            self.task_description = "".to_string();
            self.task_project = "".to_string();
            self.new_task_popup = true;
        }

        ui.allocate_space(Vec2::new(444.0, 2.0));
        self.main_container_size = Vec2::new(438.0, 170.0);

        let frame = Frame {
            fill: Color32::from_rgb(255, 228, 225),
            stroke: egui::Stroke::new(1.0, Color32::from_rgb(253, 108, 158)),
            rounding: egui::Rounding::same(2.0),
            ..Default::default()
        };

        self.update_tasks();

        let container_rect =
            egui::Rect::from_min_size(Pos2::new(7.0, 71.0), self.main_container_size);

        let tasks = self.tasks.clone(); // Clone tasks to avoid borrow issues

        ui.allocate_ui_at_rect(container_rect, |ui| {
            frame.show(ui, |ui| {
                ui.label("                       Tasks:");

                self.show_add_task_button(ui);
                let task_widget_rect =
                    Rect::from_min_size(Pos2::new(10.0, 95.0), Vec2::new(400.0, 150.0));
                ui.allocate_ui_at_rect(task_widget_rect, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.show_tasks(ui, &tasks); // Pass the cloned tasks
                        });
                });
            });
        });
    }

    fn update_tasks(&mut self) {
        if self.first_call || self.is_update {
            self.tasks = get_tasks();
            self.is_update = false;
            self.first_call = false;
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
                self.task_description = "".to_string();
                self.task_project = "".to_string();
                self.new_task_popup = true;
            }
        });
    }

    fn create_project_button(&mut self, ui: &mut egui::Ui, label: String, project: String) {
        if ui
            .add(
                egui::Button::new(label)
                    .fill(egui::Color32::from_rgb(255, 228, 225))
                    .min_size(egui::Vec2 { x: 50.0, y: 20.0 }),
            )
            .clicked()
        {
            self.project_category = project.to_string();
            self.is_update = true;
        }
    }

    fn show_tasks(&mut self, ui: &mut egui::Ui, tasks: &Vec<Task>) {
        if tasks.is_empty() {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    ui.label("There is nothing to do");
                },
            );
        } else {
            self.get_project_names();
            egui::ScrollArea::horizontal()
                .auto_shrink([true; 2])
                .max_width(400.0)
                .id_source("projects_buttons_scroll_area")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        self.create_project_button(ui, "All".to_string(), "All".to_string());
                        for project in self.project_names.clone() {
                            self.create_project_button(ui, project.clone(), project.clone());
                        }
                    });
                });

            let filtered_tasks: Vec<&Task> = if self.project_category != "All" {
                tasks
                    .iter()
                    .filter(|task| task.project == self.project_category)
                    .collect()
            } else {
                tasks.iter().collect()
            };

            if filtered_tasks.is_empty() {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label("No tasks in this project.");
                    },
                );
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .id_source("tasks_scroll_area")
                    .max_width(420.0)
                    .max_height(110.0)
                    .show(ui, |ui| {
                        for task in filtered_tasks {
                            let limit: usize = 37;
                            let description_new = if task.description.chars().count() > limit {
                                format!(
                                    " {}...",
                                    &task.description.chars().take(limit).collect::<String>()
                                )
                            } else {
                                format!(" {}", task.description.clone())
                            };
                            ui.horizontal(|ui| {
                                ui.label(description_new).on_hover_ui(|ui| {
                                    ui.label(&task.description); // Всплывающая подсказка
                                });

                                self.task_actions(ui, task.id);
                            });
                        }
                    });
            }
        }
    }
    fn task_actions(&mut self, ui: &mut egui::Ui, task_id: i32) {
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new("") // Иконка для редактирования
                        .fill(Color32::from_rgb(255, 228, 225)) // Розовый фон
                        .min_size(Vec2 { x: 16.0, y: 16.0 }),
                )
                .clicked()
            {
                self.edit_task_popup = true;
                self.current_task_id = Some(task_id);
                self.task_description = self
                    .tasks
                    .iter()
                    .find(|task| task.id == task_id)
                    .unwrap()
                    .description
                    .clone();
                self.task_project = self
                    .tasks
                    .iter()
                    .find(|task| task.id == task_id)
                    .unwrap()
                    .project
                    .clone();
                self.is_update = true;
            }

            if ui
                .add(
                    egui::Button::new("󰆴") // Иконка для удаления
                        .fill(Color32::from_rgb(255, 228, 225)) // Розовый фон
                        .min_size(Vec2 { x: 16.0, y: 16.0 }),
                )
                .clicked()
            {
                self.delete_task(task_id);
                self.is_update = true;
            }

            if ui
                .add(
                    egui::Button::new("󰄲") // Иконка для завершения задачи
                        .fill(Color32::from_rgb(255, 228, 225)) // Розовый фон
                        .min_size(Vec2 { x: 16.0, y: 16.0 }),
                )
                .clicked()
            {
                self.done_task(task_id);
                self.is_update = true;
            }
        });
    }
}
