use crate::imports::*;
use egui::{Context, Color32, Id, Pos2, TextureHandle, Ui, Vec2};
use std::time::Instant;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEXTURE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[macro_export]
macro_rules! define_animation_set {
($character:ident, $base_path:expr, $ext:expr, $($anim:ident, $frame_count:expr),+) => {
  $(
    define_animation_frames!($character _ $anim, $frame_count, concat!($base_path, "/", stringify!($character)), $ext);
  )+
};
}

#[macro_export]
macro_rules! define_animation_frames {
  ($name:ident, $frame_count:expr, $path:expr) => {
    $crate::define_animation_frames!($name, $frame_count, $path, "png");
  };  
  ($name:ident, $frame_count:expr, $path:expr, $ext:expr) => {
    const $name: AnimationFrameData = {
      const FRAME_COUNT: usize = $frame_count;
      const FRAMES: [&[u8]; FRAME_COUNT] = $crate::load_frames!($frame_count, $path, $ext);
      AnimationFrameData {
        frames: &FRAMES,
        frame_count: FRAME_COUNT,
      }
    };
  };
}

#[macro_export]
macro_rules! load_frames {
  ($frame_count:expr, $path:expr, $ext:expr) => {{
    let mut frames: [&[u8]; $frame_count] = [&[]; $frame_count];
    seq!(N in 0..$frame_count {
      frames[N] = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        $path,
        "/",
        stringify!(N),
        ".",
        $ext
      ));
    });
    frames
  }};
}

#[macro_export]
macro_rules! load_debug_paths {
  ($frame_count:expr, $path:expr, $ext:expr) => {{
    let mut paths: [&str; $frame_count] = [""; $frame_count];
    seq_macro::seq!(N in 0..$frame_count {
      paths[N] = concat!(
        env!("CARGO_MANIFEST_DIR"),
        $path,
        "/",
        stringify!(N),
        ".",
        $ext
      );
    });
    paths
  }};
}

pub(crate) use define_animation_set;
pub(crate) use define_animation_frames;

pub struct SpriteAnimation {
  frames: Vec<AnimationFrame>,
  frame_duration: f32,
  start_time: Option<Instant>,
  is_animating: bool,
  should_loop: bool,
  on_complete: Option<Box<dyn FnOnce()>>,
}

struct AnimationFrame {
  texture: TextureHandle,
  size: Vec2,
}

pub struct AnimationFrameData {
  pub frames: &'static [&'static [u8]],
  pub frame_count: usize,
}

#[derive(Default)]
pub struct SpriteAnimationBuilder {
  frame_count: usize,
  fps: f32,
  path: String,
  extension: String,
  should_loop: bool,
  on_complete: Option<Box<dyn FnOnce()>>,
}

impl SpriteAnimationBuilder {
  pub fn new() -> Self {
    Self {
      frame_count: 1,
      fps: 60.0,
      path: "resources/animation".to_string(),
      extension: "png".to_string(),
      should_loop: false,
      on_complete: None,
    }
  }

  pub fn fps(mut self, fps: f32) -> Self {
    self.fps = fps;
    self
  }

  pub fn path(mut self, path: impl Into<String>) -> Self {
    self.path = path.into();
    self
  }

  pub fn extension(mut self, ext: impl Into<String>) -> Self {
    self.extension = ext.into();
    self
  }

  pub fn looping(mut self, should_loop: bool) -> Self {
    self.should_loop = should_loop;
    self
  }

  pub fn build(self, ctx: &Context, frame_data: &AnimationFrameData) -> SpriteAnimation {
    let frames = frame_data.frames.iter().map(|bytes| {
      let image = load_image_bytes(bytes).unwrap();
      let size = Vec2::new(image.width() as f32, image.height() as f32);
      let unique_id = TEXTURE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
      let texture = ctx.load_texture(
        format!("sprite_frame_{}", unique_id),
        image,
        TextureOptions::default(),
      );
      AnimationFrame { texture, size }
    }).collect();

    SpriteAnimation {
      frames,
      frame_duration: 1.0 / self.fps,
      start_time: None,
      is_animating: false,
      should_loop: self.should_loop,
      on_complete: self.on_complete,
    }
  }
}

impl SpriteAnimation {
  pub fn animate(&mut self) {
    self.is_animating = true;
    self.start_time = Some(Instant::now());
  }

  pub fn stop(&mut self) {
    self.is_animating = false;
    self.start_time = None;
  }

  pub fn set_on_complete<F: FnOnce() + 'static>(&mut self, callback: F) {
    self.on_complete = Some(Box::new(callback));
  }

  pub fn paint(&mut self, ui: &mut Ui, position: Pos2, scale: f32) {
    let current_frame = if self.is_animating {
      if let Some(start_time) = self.start_time {
        let elapsed = start_time.elapsed().as_secs_f32();
        let total_duration = self.frame_duration * self.frames.len() as f32;
        
        if elapsed >= total_duration {
          if self.should_loop {
            self.start_time = Some(Instant::now());
            0
          } else {
            self.stop();
            0
          }
        } else {
          let frame = ((elapsed / self.frame_duration) as usize) % self.frames.len();
          frame
        }
      } else {
        0
      }
    } else {
      0
    };

    let frame = &self.frames[current_frame];
    let scaled_size = frame.size * scale;
    let rect = egui::Rect::from_min_size(position, scaled_size);

    let painter = ui.painter();
    painter.image(
      frame.texture.id(),
      rect,
      egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
      Color32::WHITE,
    );
  }

  pub fn size(&self) -> Vec2 {
    self.frames[0].size
  }

  pub fn is_animating(&self) -> bool {
    self.is_animating
  }
}

pub trait ColorAnimation  {
  fn animate_color_with_time(&self, id: Id, target: Color32, animation_time: f32) -> Color32;
}

impl ColorAnimation for Context {
  fn animate_color_with_time(&self, id: Id, target: Color32, animation_time: f32) -> Color32 {
    let current = self.data(|data| {
      data.get_temp::<Color32>(id)
        .unwrap_or(target)
    });

    // Calculate individual factors for R, G, and B
    let r_factor = self.animate_value_with_time(id.with("r"), target.r() as f32, animation_time);
    let g_factor = self.animate_value_with_time(id.with("g"), target.g() as f32, animation_time);
    let b_factor = self.animate_value_with_time(id.with("b"), target.b() as f32, animation_time);
    let a_factor = self.animate_value_with_time(id.with("a"), target.a() as f32, animation_time);

    let animated_color = Color32::from_rgba_premultiplied(
      r_factor as u8,
      g_factor as u8,
      b_factor as u8,
      a_factor as u8,
    );

    self.data_mut(|data| {
      data.insert_temp(id, animated_color);
    });

    animated_color
  }
}
