use crate::components::{Component, ComponentT};
use crate::core::Core;

pub struct Hello {
    name: String,
}

impl Default for Hello {
    fn default() -> Self {
        Self {
            name: "World".to_owned(),
        }
    }
}

impl ComponentT for Hello {
    fn name(&self) -> Option<&'static str> {
        Some("Hello")
    }

    fn render(
        &mut self,
        _core: &mut Core,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading(format!("Hello, {}!", self.name));
        ui.horizontal(|ui| {
            ui.label("Your name: ");
            ui.text_edit_singleline(&mut self.name);
        });
    }
}