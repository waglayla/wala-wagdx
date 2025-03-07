use crate::components::ComponentT;
use crate::core::Core;

pub struct Blank {}

impl Default for Blank {
    fn default() -> Self {
      Self {}
    }
}

impl ComponentT for Blank {
    fn name(&self) -> Option<&'static str> {
        Some("Blank")
    }

    fn render(
        &mut self,
        _core: &mut Core,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        ui.heading("Nothing to see here, bud");
    }
}