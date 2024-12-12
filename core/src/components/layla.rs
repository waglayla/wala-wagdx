use crate::imports::*;
use super::*;

#[macro_export]
macro_rules! load_animation_frames {
  ($name:expr, $frame_count:expr) => {{
    let mut frames = Vec::with_capacity($frame_count);
    for i in 0..$frame_count {
      frames.push(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/animation/",
        $name,
        "/",
        stringify!($i),
        ".png"
      )));
    }
    frames
  }};
}

struct LaylaFrame {
  texture: TextureHandle,
  size: Vec2,
}

struct Mascot {
  frames: Vec<MascotFrame>,
  frame_duration: f32,
  start_time: Option<Instant>,
  is_animating: bool,
}