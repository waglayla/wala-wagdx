use crate::imports::*;

use egui::{Context, TextureHandle};
use std::sync::Once;

pub struct Assets {
  pub wala_coin: TextureHandle,
  pub wala_text_logo_png: TextureHandle,
  pub paw_banner: TextureHandle,
}

static mut ASSETS: Option<Assets> = None;
static INIT: Once = Once::new();

impl Assets {
  pub fn init(ctx: &Context) {
    INIT.call_once(|| {
      let assets = Assets {
        wala_coin: Self::load_wala_coin(ctx),
        wala_text_logo_png: Self::load_wala_text_logo_png(ctx),
        paw_banner: Self::load_paw_banner_png(ctx),
      };
      unsafe {
        ASSETS = Some(assets);
      }
    });
  }

  pub fn get() -> &'static Assets {
    unsafe {
      ASSETS.as_ref().expect("Assets not initialized")
    }
  }

  fn load_wala_coin(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/wala_coin.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load WALA coin image");
    
    ctx.load_texture(
      "wala_coin",
      image,
      egui::TextureOptions::default()
    )
  }

  fn load_wala_text_logo_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/text_logo.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load WALA text logo png");
    
    ctx.load_texture(
      "wala_text_logo_png",
      image,
      egui::TextureOptions::default()
    )
  }

  fn load_paw_banner_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/paws.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load paw banner image");
    
    ctx.load_texture(
      "paw_banner",
      image,
      egui::TextureOptions::default()
    )
  }
}