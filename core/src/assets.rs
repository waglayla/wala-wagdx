use crate::imports::*;

// assets.rs
use egui::{Context, TextureHandle};
use std::sync::Once;

pub struct Assets {
    pub wala_coin: TextureHandle,
    // Add other assets here as needed
}

static mut ASSETS: Option<Assets> = None;
static INIT: Once = Once::new();

impl Assets {
    pub fn init(ctx: &Context) {
        INIT.call_once(|| {
            let assets = Assets {
                wala_coin: Self::load_wala_coin(ctx),
                // Initialize other assets here
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
}