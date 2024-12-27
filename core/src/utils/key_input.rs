use crate::imports::*;
use super::*;

pub fn handle_enter_key(ui: &Ui) -> bool {
  ui.input(|i| i.key_pressed(egui::Key::Enter))
}