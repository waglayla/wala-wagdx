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

pub fn lerp_dx(start: f32, end: f32, t: f32) -> f32 {
  start + t * (end - start)
}