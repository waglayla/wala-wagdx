use crate::imports::*;

use egui::{Context, TextureHandle};
use std::sync::Once;

pub struct Assets {
  pub wala_coin: TextureHandle,
  pub wala_text_logo_png: TextureHandle,
  pub paw_banner: TextureHandle,
  pub paw_watermark: TextureHandle,
  pub snow_watermark: TextureHandle,
  pub snow_platform: TextureHandle,
  pub meadow_watermark: TextureHandle,
  pub tree: TextureHandle,
  pub brella: TextureHandle,
  pub coins: TextureHandle,
  pub sand_watermark: TextureHandle,
  pub beach: TextureHandle,
  pub cash_watermark: TextureHandle,
  pub bark_incoming: Vec<u8>,
  pub bark_outgoing: Vec<u8>,
}

static mut ASSETS: Option<Assets> = None;
static INIT: Once = Once::new();

#[macro_export]
macro_rules! load_bytes {
  ($path:expr) => {
    include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      $path
    )).to_vec()
  };
}

impl Assets {
  pub fn init(ctx: &Context) {
    INIT.call_once(|| {
      let assets = Assets {
        wala_coin: Self::load_wala_coin(ctx),
        wala_text_logo_png: Self::load_wala_text_logo_png(ctx),
        paw_banner: Self::load_paw_banner_png(ctx),
        // TODO: use a macro for loading watermarks
        paw_watermark: Self::load_paw_watermark_png(ctx),
        snow_watermark: Self::load_snow_watermark_png(ctx),
        snow_platform: Self::load_snow_platform_png(ctx),
        meadow_watermark: Self::load_meadow_watermark_png(ctx),
        tree: Self::load_tree_png(&ctx),
        brella: Self::load_brella_png(&ctx),
        coins: Self::load_coins_png(&ctx),
        sand_watermark: Self::load_sand_watermark_png(ctx),
        beach: Self::load_beach_png(ctx),
        cash_watermark: Self::load_cash_watermark_png(ctx),
        bark_incoming: load_bytes!("/resources/sound/barks/incoming.wav"),
        bark_outgoing: load_bytes!("/resources/sound/barks/outgoing.wav"),
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
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
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
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_paw_banner_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/paw_banner.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load paw banner image");
    
    ctx.load_texture(
      "paw_banner",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_paw_watermark_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/paw_watermark.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load paw watermark image");
    
    ctx.load_texture(
      "paw_watermark",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_snow_watermark_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/snow_watermark.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load snow watermark image");
    
    ctx.load_texture(
      "snow_watermark",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_meadow_watermark_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/meadow_watermark.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load meadow watermark image");
    
    ctx.load_texture(
      "meadow_watermark",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_sand_watermark_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/sand_watermark.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load sand watermark image");
    
    ctx.load_texture(
      "sand_watermark",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_cash_watermark_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/cash_watermark.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load cash watermark image");
    
    ctx.load_texture(
      "cash_watermark",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_snow_platform_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/snow_platform.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load snow platform image");
    
    ctx.load_texture(
      "snow_platform",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_tree_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/tree.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load tree image");
    
    ctx.load_texture(
      "tree",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }

  fn load_brella_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/brella.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to load brella image");
    
    ctx.load_texture(
      "brella",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }
  fn load_coins_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/coins.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to coins brella image");
    
    ctx.load_texture(
      "brella",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }
  fn load_beach_png(ctx: &Context) -> TextureHandle {
    let image_bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/resources/images/beach.png"
    ));
    
    let image = load_image_bytes(image_bytes)
      .expect("Failed to coins beach image");
    
    ctx.load_texture(
      "beach",
      image,
      egui::TextureOptions {
        magnification: TextureFilter::Linear,
        minification: TextureFilter::Linear,
        mipmap_mode: Some(TextureFilter::Linear),
        .. Default::default()
      }
    )
  }
}