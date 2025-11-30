use glam::Vec2;

pub struct DisplayableVec2(Vec2);

impl DisplayableVec2 {
    pub fn new(vec: Vec2) -> Self { Self(vec) }
}

impl egui::Widget for &mut DisplayableVec2 {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.add(
                egui::DragValue::from_get_set(|new_value: Option<f64>| {
                    if let Some(value) = new_value {
                        self.0.x = value as f32;
                    }
                    self.0.x as f64
                })
                .speed(1.0),
            )
            .union(
                ui.add(
                    egui::DragValue::from_get_set(|new_value: Option<f64>| {
                        if let Some(value) = new_value {
                            self.0.y = value as f32;
                        }
                        self.0.y as f64
                    })
                    .speed(1.0),
                ),
            )
        })
        .response
    }
}

impl std::ops::Deref for DisplayableVec2 {
    type Target = Vec2;
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl std::ops::DerefMut for DisplayableVec2 {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}
