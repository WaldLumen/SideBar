use std::sync::{Arc, Mutex};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::fs::File;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub timestamp: String,
    pub id: u64, // Уникальный ID для удаления
}

pub struct NotificationsListener {
    notifications: Arc<Mutex<Vec<Notification>>>,
    storage_path: PathBuf,
    next_id: Arc<Mutex<u64>>,
}

impl NotificationsListener {
    pub fn new() -> Self {
        let storage_path = Self::get_storage_path();
        let (notifications, max_id) = Self::load_from_file(&storage_path);
        
        Self {
            notifications: Arc::new(Mutex::new(notifications)),
            storage_path,
            next_id: Arc::new(Mutex::new(max_id + 1)),
        }
    }

    fn get_storage_path() -> PathBuf {
        let mut path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"));
        path.push("sidebar");
        std::fs::create_dir_all(&path).ok();
        path.push("notifications.json");
        path
    }

    fn load_from_file(path: &PathBuf) -> (Vec<Notification>, u64) {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<Notification>>(&content) {
                    Ok(notifs) => {
                        let max_id = notifs.iter().map(|n| n.id).max().unwrap_or(0);
                        println!("Loaded {} notifications from file", notifs.len());
                        (notifs, max_id)
                    }
                    Err(e) => {
                        eprintln!("Failed to parse notifications: {}", e);
                        (Vec::new(), 0)
                    }
                }
            }
            Err(_) => {
                println!("No existing notifications file, starting fresh");
                (Vec::new(), 0)
            }
        }
    }

    fn save_to_file(&self) {
        if let Ok(notifs) = self.notifications.lock() {
            match serde_json::to_string_pretty(&*notifs) {
                Ok(json) => {
                    if let Ok(mut file) = File::create(&self.storage_path) {
                        let _ = file.write_all(json.as_bytes());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize notifications: {}", e);
                }
            }
        }
    }

    pub fn get_notifications(&self) -> Arc<Mutex<Vec<Notification>>> {
        Arc::clone(&self.notifications)
    }

    pub fn start_listening(&self, ctx: egui::Context) {
        let notifications = Arc::clone(&self.notifications);
        let storage_path = self.storage_path.clone();
        let next_id = Arc::clone(&self.next_id);
        
        std::thread::spawn(move || {
            Self::listen_loop(notifications, ctx, storage_path, next_id);
        });
    }

    fn listen_loop(
        notifications: Arc<Mutex<Vec<Notification>>>,
        ctx: egui::Context,
        storage_path: PathBuf,
        next_id: Arc<Mutex<u64>>,
    ) {
        println!("Starting notification listener...");
        
        loop {
            match Self::run_dbus_monitor(&notifications, &ctx, &storage_path, &next_id) {
                Ok(_) => {
                    eprintln!("dbus-monitor exited unexpectedly, restarting...");
                }
                Err(e) => {
                    eprintln!("Failed to start dbus-monitor: {}, retrying in 5s...", e);
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }
    }

    fn run_dbus_monitor(
        notifications: &Arc<Mutex<Vec<Notification>>>,
        ctx: &egui::Context,
        storage_path: &PathBuf,
        next_id: &Arc<Mutex<u64>>,
    ) -> Result<(), String> {
        let mut child = Command::new("dbus-monitor")
            .arg("--session")
            .arg("interface='org.freedesktop.Notifications',member='Notify'")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn dbus-monitor: {}", e))?;
        
        let stdout = child.stdout.take()
            .ok_or("Failed to get stdout")?;
        let reader = BufReader::new(stdout);
        
        let mut app_name = String::new();
        let mut summary = String::new();
        let mut body = String::new();
        let mut field_count = 0;
        let mut in_method_call = false;
        
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            let line = line.trim();
            
            if line.contains("method call") && line.contains("Notify") {
                in_method_call = true;
                field_count = 0;
                app_name.clear();
                summary.clear();
                body.clear();
                println!("=== New notification detected ===");
                continue;
            }
            
            if !in_method_call {
                continue;
            }
            
            if line.starts_with("string \"") {
                let value = line
                    .trim_start_matches("string \"")
                    .trim_end_matches('"')
                    .to_string();
                
                match field_count {
                    0 => {
                        app_name = value.clone();
                        println!("Field #0 (app_name): {}", value);
                    }
                    2 => {
                        println!("Field #2 (app_icon): {}", value);
                    }
                    3 => {
                        summary = value.clone();
                        println!("Field #3 (summary): {}", value);
                    }
                    4 => {
                        body = value.clone();
                        println!("Field #4 (body): {}", value);
                    }
                    _ => {
                        println!("Field #{}: {}", field_count, value);
                    }
                }
                
                field_count += 1;
            } else if line.starts_with("uint32") || line.starts_with("int32") {
                println!("Field #{}: (integer)", field_count);
                field_count += 1;
            } else if line.starts_with("array [") {
                println!("Field #{}: (array)", field_count);
                field_count += 1;
                
                if field_count > 4 && (!app_name.is_empty() || !summary.is_empty()) {
                    println!("\nParsed notification:");
                    println!("  App: '{}'", app_name);
                    println!("  Summary: '{}'", summary);
                    println!("  Body: '{}'", body);
                    println!("===========================\n");
                    
                    let id = {
                        let mut id_lock = next_id.lock().unwrap();
                        let current_id = *id_lock;
                        *id_lock += 1;
                        current_id
                    };
                    
                    let notification = Notification {
                        app_name: app_name.clone(),
                        summary: summary.clone(),
                        body: body.clone(),
                        timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
                        id,
                    };
                    
                    if let Ok(mut notifs) = notifications.lock() {
                        notifs.push(notification);
                        if notifs.len() > 100 {
                            notifs.remove(0);
                        }
                        
                        drop(notifs);
                        Self::save_notifications_to_file(notifications, storage_path);
                    }
                    
                    ctx.request_repaint();
                    in_method_call = false;
                }
            } else if line.starts_with("dict entry(") {
                continue;
            }
        }
        
        Ok(())
    }

    fn save_notifications_to_file(
        notifications: &Arc<Mutex<Vec<Notification>>>,
        storage_path: &PathBuf,
    ) {
        if let Ok(notifs) = notifications.lock() {
            match serde_json::to_string_pretty(&*notifs) {
                Ok(json) => {
                    if let Ok(mut file) = File::create(storage_path) {
                        let _ = file.write_all(json.as_bytes());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to serialize notifications: {}", e);
                }
            }
        }
    }

    pub fn remove_notification(&self, id: u64) {
        if let Ok(mut notifs) = self.notifications.lock() {
            notifs.retain(|n| n.id != id);
        }
        self.save_to_file();
    }

    pub fn clear_all(&self) {
        if let Ok(mut notifs) = self.notifications.lock() {
            notifs.clear();
        }
        self.save_to_file();
    }

    pub fn get_count(&self) -> usize {
        self.notifications.lock().map(|n| n.len()).unwrap_or(0)
    }
}

impl Default for NotificationsListener {
    fn default() -> Self {
        Self::new()
    }
}