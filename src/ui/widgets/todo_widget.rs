use serde_json::Value;
use std::process::Command;
use std::str;

#[derive(Clone)]
pub struct Task {
    pub description: String,
    pub project: String,
    pub id: i32,
}

pub fn get_tasks() -> Vec<Task> {
    // Выполняем команду `task` для экспорта задач в формате JSON
    let output = Command::new("task")
        .args(["+PENDING", "export"])
        .output()
        .expect("Failed to execute 'task' command");

    // Преобразуем вывод команды в строку
    let json_str = str::from_utf8(&output.stdout).expect("Failed to convert output to string");

    // Парсим строку как JSON
    let parsed: Vec<Value> = serde_json::from_str(json_str).expect("Failed to parse JSON");

    let mut tasks = Vec::new();

    // Проходим по каждой задаче в JSON-массиве
    for task_data in parsed {
        // Извлекаем описание задачи
        let description = task_data["description"].as_str().unwrap_or("").to_string();

        // Извлекаем проект задачи
        let project = task_data["project"].as_str().unwrap_or("").to_string();

        // Извлекаем ID задачи
        let id = task_data["id"].as_i64().unwrap_or(0) as i32;

        // Проверяем, что проект не пустой, прежде чем добавлять задачу
        if !project.is_empty() {
            let task = Task {
                description,
                project,
                id,
            };
            tasks.push(task);
        }
    }

    tasks
}
