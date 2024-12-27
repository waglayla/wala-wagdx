use crate::imports::*;
use image::ImageOutputFormat;
use super::*;

use qrcode::render::svg;
use qrcode::*;
use egui::{ColorImage, TextureHandle, IconData};
use egui_extras::RetainedImage;

pub fn generate_qr_code_svg(input: String) -> Result<String> {
  use qrcode::{QrCode, render::svg};

  let qr = QrCode::new(input.as_bytes())
    .map_err(|e| Error::custom(e.to_string()))?;
  
  let svg = qr.render::<svg::Color>()
    .quiet_zone(false)
    .min_dimensions(200, 200)  // Set minimum dimensions
    .dark_color(svg::Color("#000000"))
    .light_color(svg::Color("#ffffff"))
    .build();

  Ok(svg)
}

// fn generate_qr_code_png(input: String) -> Result<Vec<u8>> {
//   use qrcode::QrCode;
//   use image::Luma;

//   let code = QrCode::new(input.as_bytes())?;
//   let image = code.render::<Luma<u8>>()
//     .min_dimensions(640, 640)
//     .build();
  
//   let mut png_data = Vec::new();
//   image.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageOutputFormat::Png)?;
  
//   Ok(png_data)
// }