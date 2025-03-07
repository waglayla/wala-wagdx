pub use crate::imports::*;

pub struct DXImage;

fn paint(ui: &mut Ui, texture: &TextureHandle, rect: Rect) {
  ui.painter().image(
    texture.id(),
    rect,
    Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
    Color32::WHITE,
  );
}

impl DXImage {
  pub fn draw(ui: &mut Ui, texture: &TextureHandle, width: f32, position: egui::Pos2, alignment: egui::Align2) {
    let texture_aspect_ratio = texture.size()[0] as f32 / texture.size()[1] as f32;

    let height = width / texture_aspect_ratio;
    
    let size = egui::vec2(width, height);
    let rect = match alignment {
      egui::Align2::LEFT_TOP => egui::Rect::from_min_size(position, size),
      egui::Align2::CENTER_TOP => egui::Rect::from_min_size(
        position - egui::vec2(size.x / 2.0, 0.0),
        size,
      ),
      egui::Align2::RIGHT_TOP => egui::Rect::from_min_size(
        position - egui::vec2(size.x, 0.0),
        size,
      ),
      egui::Align2::LEFT_CENTER => egui::Rect::from_min_size(
        position - egui::vec2(0.0, size.y / 2.0),
        size,
      ),
      egui::Align2::CENTER_CENTER => egui::Rect::from_min_size(
        position - egui::vec2(size.x / 2.0, size.y / 2.0),
        size,
      ),
      egui::Align2::RIGHT_CENTER => egui::Rect::from_min_size(
        position - egui::vec2(size.x, size.y / 2.0),
        size,
      ),
      egui::Align2::LEFT_BOTTOM => egui::Rect::from_min_size(
        position - egui::vec2(0.0, size.y),
        size,
      ),
      egui::Align2::CENTER_BOTTOM => egui::Rect::from_min_size(
        position - egui::vec2(size.x / 2.0, size.y),
        size,
      ),
      egui::Align2::RIGHT_BOTTOM => egui::Rect::from_min_size(
        position - egui::vec2(size.x, size.y),
        size,
      ),
    };

    let response = ui.allocate_rect(rect, egui::Sense::hover());
    paint(ui, texture, rect);
  }

  pub fn paint_at(ui: &mut Ui, texture: &TextureHandle, width: f32, position: egui::Pos2, alignment: egui::Align2) {
    let texture_aspect_ratio = texture.size()[0] as f32 / texture.size()[1] as f32;

    let height = width / texture_aspect_ratio;
    
    let size = egui::vec2(width, height);
    let rect = match alignment {
      egui::Align2::LEFT_TOP => egui::Rect::from_min_size(position, size),
      egui::Align2::CENTER_TOP => egui::Rect::from_min_size(
        position - egui::vec2(size.x / 2.0, 0.0),
        size,
      ),
      egui::Align2::RIGHT_TOP => egui::Rect::from_min_size(
        position - egui::vec2(size.x, 0.0),
        size,
      ),
      egui::Align2::LEFT_CENTER => egui::Rect::from_min_size(
        position - egui::vec2(0.0, size.y / 2.0),
        size,
      ),
      egui::Align2::CENTER_CENTER => egui::Rect::from_min_size(
        position - egui::vec2(size.x / 2.0, size.y / 2.0),
        size,
      ),
      egui::Align2::RIGHT_CENTER => egui::Rect::from_min_size(
        position - egui::vec2(size.x, size.y / 2.0),
        size,
      ),
      egui::Align2::LEFT_BOTTOM => egui::Rect::from_min_size(
        position - egui::vec2(0.0, size.y),
        size,
      ),
      egui::Align2::CENTER_BOTTOM => egui::Rect::from_min_size(
        position - egui::vec2(size.x / 2.0, size.y),
        size,
      ),
      egui::Align2::RIGHT_BOTTOM => egui::Rect::from_min_size(
        position - egui::vec2(size.x, size.y),
        size,
      ),
    };

    paint(ui, texture, rect);
  }
}