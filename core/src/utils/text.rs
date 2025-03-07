use egui::{FontFamily, FontId};

pub fn calculate_text_width(
  text: &str, 
  ctx: &egui::Context, 
  font_size: f32, 
  font_family: FontFamily
) -> f32 {
  ctx.fonts(|fonts| {
    let font = FontId::new(font_size, font_family);
    let text_galley  = fonts.layout_no_wrap(
      text.to_string(),
      font,
      egui::Color32::WHITE,
    );
    text_galley .rect.width()
  })
}