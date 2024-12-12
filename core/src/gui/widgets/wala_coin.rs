pub use crate::imports::*;
pub use super::*;

pub struct WalaCoin;

impl WalaCoin {
  pub fn render(ui: &mut Ui, rect: Rect) {
    let response = ui.allocate_rect(rect, egui::Sense::hover());
    WalaCoin::paint(ui, rect);
  }

  pub fn paint(ui: &mut Ui, rect: Rect) {
    let texture = &Assets::get().wala_coin;
    ui.painter().image(
      texture.id(),
      rect,
      Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
      Color32::WHITE,
    );
  }
}