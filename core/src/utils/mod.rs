use crate::imports::*;

mod i18n;
#[cfg(not(target_arch = "wasm32"))]
pub use i18n::*;

mod color;
pub use color::*;
mod arglist;
pub use arglist::*;
mod qr;
pub use qr::*;
mod get_public;
pub use get_public::*;
mod text;
pub use text::*;
mod dx_image;
pub use dx_image::*;
mod mass;
pub use mass::*;
mod sync;
pub use sync::*;
mod key_input;
pub use key_input::*;
mod enum_macros;
pub use enum_macros::*;
mod focus;
pub use focus::*;
mod format;
pub use format::*;
mod sound;
pub use sound::*;

pub fn lerp_dx(start: f32, end: f32, t: f32) -> f32 {
  start + t * (end - start)
}