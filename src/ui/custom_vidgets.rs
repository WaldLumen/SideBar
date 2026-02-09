use egui::{Color32, ImageButton, Response, Ui, Vec2, Image, TextureHandle};

pub struct StyledImageButton<'a> {
    texture: &'a TextureHandle,
    size: Vec2,
    bg_color: Color32,
    rounding: f32,
}

impl<'a> StyledImageButton<'a> {
    pub fn new(texture: &'a TextureHandle) -> Self {
        Self {
            texture,
            size: Vec2::new(15.0, 10.0),
            bg_color: Color32::from_rgb(50, 50, 50),
            rounding: 5.0,
        }
    }
    
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    
    pub fn bg_color(mut self, color: Color32) -> Self {
        self.bg_color = color;
        self
    }
    
    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }
    
    pub fn show(self, ui: &mut Ui) -> Response {
    ui.scope(|ui| {
        let rounding = egui::Rounding::same(self.rounding);
        
        // Получаем изменяемую ссылку один раз
        let style = ui.style_mut();
        let widgets = &mut style.visuals.widgets;
        
        // Модифицируем каждый виджет отдельно
        widgets.inactive.weak_bg_fill = self.bg_color;
        widgets.inactive.bg_fill = self.bg_color;
        widgets.inactive.rounding = rounding;
        
        widgets.hovered.weak_bg_fill = self.bg_color;
        widgets.hovered.bg_fill = self.bg_color;
        widgets.hovered.rounding = rounding;
        
        widgets.active.weak_bg_fill = self.bg_color;
        widgets.active.bg_fill = self.bg_color;
        widgets.active.rounding = rounding;
        
        ui.add(
            ImageButton::new(
                Image::new(self.texture).fit_to_exact_size(self.size)
            )
            .frame(true)
            .rounding(rounding) // Добавьте эту строку
        )
    }).inner
}
}