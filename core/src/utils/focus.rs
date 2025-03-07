use super::*;

pub fn set_focus(ui: &mut Ui, next: egui::Response) {
  ui.ctx().memory_mut(|mem| mem.request_focus(next.id));
}