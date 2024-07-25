use std::process::{Command, Stdio};
use std::str;

#[derive(Clone)]
pub struct Task {
    pub description: String,
    pub project: String,
    pub id: i32,
}

pub fn get_tasks() -> Vec<Task> {
    let max_id_raw = Command::new("task")
        .args(["+PENDING", "count"])
        .output()
        .expect("Failed to execute 'task' command");

    let max_id_str = str::from_utf8(&max_id_raw.stdout)
        .expect("Failed to convert output to string");

    let max_id_int: i32 = max_id_str.trim().parse().expect("Failed to parse max ID");

    let mut tasks = Vec::new();

    for id in 1..=max_id_int {
        let task_info = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "task {} info | grep -A 1 '^Description' | grep -v '^Status' | cut -d ' ' -f 2- | tr -s ' '",
                id
            ))
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute 'task' command");

        let task_description = str::from_utf8(&task_info.stdout)
            .expect("Failed to convert output to string")
            .trim()
            .to_string();

        let raw_task_project = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "task {} info | grep -A 0 '^Project' | grep -v '^Status' | cut -d ' ' -f 2- | tr -s ' '",
                id
            ))
            .stdout(Stdio::piped())
            .output()
            .expect("Failed to execute 'task' command");

        let task_project = str::from_utf8(&raw_task_project.stdout)
            .expect("Failed to convert output to string")
            .trim()
            .to_string();

        let task = Task {
            description: task_description,
            project: task_project,
            id,
        };

        tasks.push(task);
    }

    tasks
}
