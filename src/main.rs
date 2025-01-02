mod ui;

use ui::sidebar::SideBar;

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),

        ..Default::default()
    };
    let _app = eframe::run_native(
        "SideBar",
        native_options,
        Box::new(|cc| Box::new(SideBar::new(cc))),
    );
}
