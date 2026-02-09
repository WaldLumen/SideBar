use crate::ui::color_parser::parse_color_from_ini;
use crate::ui::widgets::todo_widget::{get_tasks, Task};
use egui::{Frame, Key, TextEdit, Vec2, Window};
use std::process::Command;
use image::GenericImageView;
use crate::ui::custom_vidgets::StyledImageButton;

pub(crate) struct TaskManager {
    pub tasks: Vec<Task>,
    pub task_project: String,
    pub task_description: String,
    pub project_category: String,
    pub project_names: Vec<String>,
    pub current_task_id: Option<i32>,
    pub new_task_popup: bool,
    pub edit_task_popup: bool,
    pub first_call: bool,
    pub is_update: bool,
    // Кэшируем текстуру чтобы не загружать каждый кадр
    add_icon_texture: Option<egui::TextureHandle>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: Vec::default(),
            task_project: String::default(),
            task_description: String::default(),
            project_category: "All".to_string(),
            current_task_id: Some(i32::default()),
            new_task_popup: false,
            edit_task_popup: false,
            first_call: true,
            is_update: false,
            project_names: Vec::default(),
            add_icon_texture: None,
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
                .fixed_size(Vec2::new(300.0, 200.0))
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("New task:");
                        ui.add(
                            TextEdit::multiline(&mut self.task_description)
                                .desired_width(280.0)
                                .desired_rows(3),
                        );
                        
                        ui.add_space(5.0);
                        
                        ui.label("Project:");
                        ui.add(
                            TextEdit::singleline(&mut self.task_project)
                                .desired_width(280.0),
                        );
                        
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new("Save")
                                        .min_size(Vec2 { x: 80.0, y: 30.0 })
                                        .fill(parse_color_from_ini("button-color")),
                                )
                                .clicked()
                            {
                                self.new_task_popup = false;
                                self.add_task();
                                self.is_update = true;
                            }

                            if ui
                                .add(
                                    egui::Button::new("Close")
                                        .min_size(Vec2 { x: 80.0, y: 30.0 })
                                        .fill(parse_color_from_ini("button-color")),
                                )
                                .clicked()
                            {
                                self.new_task_popup = false;
                            }
                        });
                    });
                });
        }
    }

    pub fn edit_task_popup(&mut self, ctx: &egui::Context) {
        if self.edit_task_popup {
            Window::new("Edit Task")
                .title_bar(false)
                .collapsible(false)
                .resizable(false)
                .fixed_size(Vec2::new(300.0, 200.0))
                .show(ctx, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Edit task:");
                        ui.add(
                            TextEdit::multiline(&mut self.task_description)
                                .desired_width(280.0)
                                .desired_rows(3),
                        );

                        ui.add_space(5.0);

                        ui.label("Project:");
                        ui.add(
                            TextEdit::singleline(&mut self.task_project)
                                .desired_width(280.0),
                        );
                        
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new("Save")
                                        .min_size(Vec2 { x: 80.0, y: 30.0 })
                                        .fill(parse_color_from_ini("button-color")),
                                )
                                .clicked()
                            {
                                self.edit_task_popup = false;
                                self.modify_task();
                                self.is_update = true;
                            }

                            if ui
                                .add(
                                    egui::Button::new("Close")
                                        .min_size(Vec2 { x: 80.0, y: 30.0 })
                                        .fill(parse_color_from_ini("button-color")),
                                )
                                .clicked()
                            {
                                self.edit_task_popup = false;
                            }
                        });
                    });
                });
        }
    }

    pub fn show_tasks_widget(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.handle_keyboard_shortcuts(ctx);
        self.update_tasks();
        self.load_texture_if_needed(ctx);
        self.render_task_frame(ui, ctx);
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        let input = ctx.input(|i| i.clone());
        if input.key_pressed(Key::N) && input.modifiers.ctrl {
            self.open_new_task_dialog();
        }
    }

    fn open_new_task_dialog(&mut self) {
        self.task_description.clear();
        self.task_project.clear();
        self.new_task_popup = true;
    }

    fn update_tasks(&mut self) {
        if self.first_call || self.is_update {
            self.tasks = get_tasks();
            self.is_update = false;
            self.first_call = false;
        }
    }

    fn load_texture_if_needed(&mut self, ctx: &egui::Context) {
        if self.add_icon_texture.is_none() {
            let img = image::open("/home/rika/code/SideBar-Rust/src/assets/icons/add-item.png")
                .expect("Failed to load add-item icon");
            let size = [img.width() as usize, img.height() as usize];
            let image_buffer = img.to_rgba8();
            let pixels = image_buffer.as_flat_samples();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            let texture = ctx.load_texture("add_task_icon", color_image, egui::TextureOptions::default());
            self.add_icon_texture = Some(texture);
        }
    }

    fn render_task_frame(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let frame = self.create_frame();
        
        ui.vertical(|ui| {
            frame.show(ui, |ui| {
                self.render_header(ui, ctx);
                ui.add_space(5.0);
                ui.separator();
                ui.add_space(5.0);
                self.render_task_list(ui);
            });
        });
    }

    fn create_frame(&self) -> Frame {
        Frame {
            fill: parse_color_from_ini("frame-background"),
            stroke: egui::Stroke::new(1.0, parse_color_from_ini("frame-border-color")),
            rounding: egui::Rounding::same(4.0),
            inner_margin: egui::Margin::same(12.0),
            ..Default::default()
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.heading("Tasks");
            
            // Выравниваем кнопку вправо
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.show_add_task_button(ui, ctx);
            });
        });
    }

    fn render_task_list(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height().min(200.0);
        
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .max_height(available_height)
            .show(ui, |ui| {
                let tasks = self.tasks.clone();
                self.show_tasks(ui, &tasks);
            });
    }

    fn show_add_task_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if let Some(texture) = &self.add_icon_texture {
            if StyledImageButton::new(texture)
                .size(Vec2::new(20.0, 20.0))
                .bg_color(parse_color_from_ini("button-color"))
                .rounding(4.0)
                .show(ui)
                .clicked()
            {
                self.open_new_task_dialog();
            }
        }
    }

    fn create_project_button(&mut self, ui: &mut egui::Ui, label: String, project: String) {
        let is_selected = self.project_category == project;
        let button_color = if is_selected {
            parse_color_from_ini("button-color").linear_multiply(1.3) // Подсветка выбранной категории
        } else {
            parse_color_from_ini("button-color")
        };

        if ui
            .add(
                egui::Button::new(&label)
                    .min_size(egui::Vec2 { x: 50.0, y: 25.0 })
                    .fill(button_color),
            )
            .clicked()
        {
            self.project_category = project;
            self.is_update = true;
        }
    }

    fn show_tasks(&mut self, ui: &mut egui::Ui, tasks: &Vec<Task>) {
        if tasks.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("📭 There is nothing to do");
            });
            return;
        }

        self.get_project_names();
        
        // Кнопки фильтрации проектов
        ui.horizontal_wrapped(|ui| {
            self.create_project_button(ui, "All".to_string(), "All".to_string());
            for project in self.project_names.clone() {
                self.create_project_button(ui, project.clone(), project.clone());
            }
        });

        ui.add_space(10.0);

        let filtered_tasks: Vec<&Task> = if self.project_category != "All" {
            tasks
                .iter()
                .filter(|task| task.project == self.project_category)
                .collect()
        } else {
            tasks.iter().collect()
        };

        if filtered_tasks.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.label("📂 No tasks in this project");
            });
        } else {
            self.render_tasks_grid(ui, &filtered_tasks);
        }
    }

    fn render_tasks_grid(&mut self, ui: &mut egui::Ui, tasks: &[&Task]) {
        egui::Grid::new("tasks_grid")
            .striped(true)
            .spacing([18.0, 8.0])
            .min_col_width(300.0)
            .show(ui, |ui| {
                for task in tasks {
                    self.render_task_row(ui, task);
                    ui.end_row();
                }
            });
    }

    fn render_task_row(&mut self, ui: &mut egui::Ui, task: &Task) {
        const DESCRIPTION_LIMIT: usize = 50;
        
        let description_display = if task.description.chars().count() > DESCRIPTION_LIMIT {
            format!(
                "{}...",
                task.description.chars().take(DESCRIPTION_LIMIT).collect::<String>()
            )
        } else {
            task.description.clone()
        };

        ui.label(&description_display)
            .on_hover_text(&task.description);
        
        self.task_actions(ui, task);
    }

    fn task_actions(&mut self, ui: &mut egui::Ui, task: &Task) {
        ui.horizontal(|ui| {
            // Кнопка редактирования
            if ui
                .add(
                    egui::Button::new("✏")
                        .min_size(Vec2 { x: 24.0, y: 24.0 })
                        .fill(parse_color_from_ini("button-color")),
                )
                .on_hover_text("Edit task")
                .clicked()
            {
                self.edit_task_popup = true;
                self.current_task_id = Some(task.id);
                self.task_description = task.description.clone();
                self.task_project = task.project.clone();
                self.is_update = true;
            }

            // Кнопка удаления
            if ui
                .add(
                    egui::Button::new("🗑")
                        .min_size(Vec2 { x: 24.0, y: 24.0 })
                        .fill(parse_color_from_ini("button-color")),
                )
                .on_hover_text("Delete task")
                .clicked()
            {
                self.delete_task(task.id);
                self.is_update = true;
            }

            // Кнопка завершения
            if ui
                .add(
                    egui::Button::new("✓")
                        .min_size(Vec2 { x: 24.0, y: 24.0 })
                        .fill(parse_color_from_ini("button-color")),
                )
                .on_hover_text("Complete task")
                .clicked()
            {
                self.done_task(task.id);
                self.is_update = true;
            }
        });
    }
}