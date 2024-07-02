// TODO:
// 1. Добавить возможность добавлять и убирать таски
// 2. Добавить возможность насттраивать теги и проекты
// 3. Добавить возможность стартовать и стопать таски.

use std::process::{Command, Stdio};
use std::str;

pub fn get_tasks() -> Vec<Vec<String>> {
    // Количество тасков, оно же количество итераций цикла ниже
    let max_id_raw = Command::new("task")
        .args(["+PENDING", "count"])
        .output()
        .expect("Failed to execute 'task' command");

    let max_id_str = str::from_utf8(&max_id_raw.stdout)
        .expect("Failed to convert output to string");

    let max_id_int: i32 = max_id_str.trim().parse().expect("Failed to parse max ID");

    //Получаем описания каждого таска
    let mut tasks: Vec<Vec<String>> = Vec::new();

    for id in 1..=max_id_int {
        let task_info = Command::new("sh")
            .arg("-c")
            .arg(format!("task {} info | grep '^Description' | cut -d ' ' -f 2-", id))
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute 'task' command");

        let task_description = str::from_utf8(&task_info.stdout)
            .expect("Failed to convert output to string");

        let mut task: Vec<String> = Vec::new();
        task.push(task_description.trim().to_string());

        tasks.push(task);
    }

    tasks // Return the vector of tasks
}



