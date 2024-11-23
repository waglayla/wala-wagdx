use egui::Color32;

pub trait Color32Extension {
  fn from_f32(value: f32) -> Self;
  fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self;
  fn to_hex(&self) -> String;
  fn linear_multiply_rgb(&self, factor: f32) -> Self;
}

impl Color32Extension for Color32 {
  fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
    Color32::from_rgba_premultiplied(r, g, b, a)
  }
  fn from_f32(value: f32) -> Self {
    Color32::from_rgba_premultiplied(
      (value * 255.0) as u8,
      (value * 255.0) as u8,
      (value * 255.0) as u8,
      (value * 255.0) as u8,
    )
  }
  fn to_hex(&self) -> String {
    format!(
      "#{:02X}{:02X}{:02X}{:02X}",
      self.r(),
      self.g(),
      self.b(),
      self.a()
    )
  }
  fn linear_multiply_rgb(&self, factor: f32) -> Self {
    let darken = |c: u8| -> u8 {
        let f = f32::from(c) / 255.0;
        let darkened = f * factor;
        (darkened * 255.0).round() as u8
    };

    Color32::from_rgba_premultiplied(darken(self.r()),darken(self.g()),darken(self.b()),self.a())
  }
}