pub use egui::SizeHint;
pub use egui_extras::image::{load_svg_bytes, load_svg_bytes_with_size};

pub fn color_image_to_icon_data(image: epaint::ColorImage) -> egui::IconData {
  egui::IconData {
    width: image.size[0] as u32,
    height: image.size[1] as u32,
    rgba: image.as_raw().to_vec(),
  }
}

pub fn svg_to_icon_data(svg_bytes: &[u8], size_hint: Option<SizeHint>) -> egui::IconData {
  let image = load_svg_bytes_with_size(svg_bytes, size_hint).unwrap();
  color_image_to_icon_data(image)
}

pub fn load_image_bytes(bytes: &[u8]) -> Result<epaint::ColorImage, String> {
  let image = image::load_from_memory(bytes)
      .map_err(|e| format!("Failed to load image: {}", e))?;
  let image_buffer = image.to_rgba8();
  let size = [image.width() as _, image.height() as _];
  let pixels = image_buffer.as_flat_samples();
  Ok(epaint::ColorImage::from_rgba_unmultiplied(
    size,
    pixels.as_slice(),
  ))
}