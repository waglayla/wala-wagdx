use crate::imports::*;
use std::borrow::Cow;
use egui_extras::image::RetainedImage;

pub struct Qr;

impl Qr {
  // UI version
  pub fn render(ui: &mut Ui, bytes: &[u8], uri: &str, rect: Rect) {
    let size = rect.width().min(rect.height());
    let square_rect = egui::Rect::from_min_size(
      rect.center() - egui::vec2(size / 2.0, size / 2.0),
      egui::vec2(size, size)
    );

    match RetainedImage::from_svg_bytes(uri, bytes) {
      Ok(image) => {
        ui.allocate_ui_at_rect(square_rect, |ui| {
          image.show_size(ui, egui::vec2(size, size));
        });
      },
      Err(err) => {
        eprintln!("Failed to load QR code: {}", err);
        ui.allocate_ui_at_rect(square_rect, |ui| {
          ui.label("QR Code Error");
        });
      }
    }
  }

  // Painter version
  // pub fn render_with_painter(painter: &Painter, bytes: load::Bytes, uri: &str, rect: Rect) {
  //   let size = rect.width().min(rect.height());
  //   let square_rect = egui::Rect::from_min_size(
  //     rect.center() - egui::vec2(size / 2.0, size / 2.0),
  //     egui::vec2(size, size)
  //   );

  //   let texture_handle = painter.ctx().load_texture(
  //     uri,
  //     load_image_bytes(bytes).unwrap(),
  //     TextureOptions::NEAREST
  //   );

  //   painter.image(
  //     texture_handle.id(),
  //     square_rect,
  //     egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), // UV coords
  //     egui::Color32::WHITE,
  //   );
  // }
}
