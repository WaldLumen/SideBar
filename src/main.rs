//use eframe::icon_data::from_png_bytes; 
//use std::process::Command;
//use egui::{Frame, Pos2, TextEdit, Vec2, Window};

mod ui;

use ui::sidebar::SideBar;
use eframe;

fn main() { 
    let native_options = eframe::NativeOptions::default();
    let _app = eframe::run_native(
        "SideBar",
        native_options,
        Box::new(|cc| Box::new(SideBar::new(cc))),
    );
}

