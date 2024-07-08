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

