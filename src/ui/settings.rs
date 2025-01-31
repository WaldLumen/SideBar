use crate::ui::color_parser::parse_color_from_ini;
use egui::{Frame, Pos2, TextEdit, Vec2, Window};
use std::process::Command;

#[derive(Default)]
pub(crate) struct Settings {
    pub theme: String,
    pub owm_api_key: String,
    pub city: String,
}

impl Settings {}
