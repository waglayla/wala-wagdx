use super::*;
use crate::components::ComponentT;
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
    ui.heading(i18n_args(
      "Hello, {name}!",
      &[("name", self.name.as_str())]
    ));
    ui.horizontal(|ui| {
      i18n("Your name: ");
      ui.text_edit_singleline(&mut self.name);
    });
  }
}