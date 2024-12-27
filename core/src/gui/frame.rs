use crate::imports::*;
use std::sync::atomic::{AtomicBool, Ordering};

static HAS_BEEN_FULLSCREEN: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
const WIN32: bool = true;
#[cfg(not(target_os = "windows"))]
const WIN32: bool = false;

pub const WINDOW_ROUNDING: f32 = 10.0;

fn style_setup(ctx: &egui::Context) {
  ctx.style_mut(|style| {
    style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke {
      color: theme_color().separator_color,
      width: 1.0,
    };
    style.visuals.override_text_color = Some(theme_color().default_color);
    style.visuals.widgets.inactive.weak_bg_fill = theme_color().button_color;
    style.visuals.widgets.hovered.weak_bg_fill = theme_color().button_color;
    style.visuals.widgets.inactive.bg_fill = theme_color().button_color; 
  });
}

pub fn create_custom_frame(
  ctx: &egui::Context,
) -> egui::Frame {
  let mut stroke = ctx.style().visuals.widgets.noninteractive.fg_stroke;
  stroke.width = 0.0;
  stroke.color = theme_color().separator_color;

  style_setup(ctx);

  egui::Frame {
    fill: theme_color().fg_color,
    rounding: 0.0.into(),
    stroke,
    ..Default::default()
  }
}

pub fn dx_shadow() -> egui::epaint::Shadow {
  egui::epaint::Shadow {
    offset: (2.0, 2.0).into(),
    color: egui::Color32::from_black_alpha(60),
    spread: 1.0,
    blur: 6.0,
    ..Default::default()
  }
}

pub fn create_custom_popup(
  ctx: &egui::Context,
) -> egui::Frame {
  let mut stroke = ctx.style().visuals.widgets.noninteractive.fg_stroke;
  stroke.width = 0.5;

  style_setup(ctx);

  egui::Frame {
    fill: theme_color().fg_color,
    shadow: dx_shadow(),
    inner_margin: egui::Margin::same(8.0),
    rounding: 10.0.into(),
    stroke,
    ..Default::default()
  }
}

fn calculate_resizable_borders_and_corners(
  app_rect: egui::Rect,
  border_thickness: f32,
) -> Option<(
  egui::Rect, // North border
  egui::Rect, // South border
  egui::Rect, // West border
  egui::Rect, // East border
  egui::Rect, // Top-left corner
  egui::Rect, // Top-right corner
  egui::Rect, // Bottom-left corner
  egui::Rect, // Bottom-right corner
)> {
  if border_thickness <= 0.0 {
    return None; // Resizing is disabled
  }

  // Corner rectangles
  let top_left = egui::Rect::from_min_size(app_rect.min, egui::vec2(border_thickness, border_thickness));
  let top_right = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(app_rect.width() - border_thickness, 0.0),
    egui::vec2(border_thickness, border_thickness),
  );
  let bottom_left = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(0.0, app_rect.height() - border_thickness),
    egui::vec2(border_thickness, border_thickness),
  );
  let bottom_right = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(app_rect.width() - border_thickness, app_rect.height() - border_thickness),
    egui::vec2(border_thickness, border_thickness),
  );

  // Border rectangles (trimmed to avoid overlap with corners)
  let north_rect = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(border_thickness, 0.0),
    egui::vec2(app_rect.width() - 2.0 * border_thickness, border_thickness),
  );
  let south_rect = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(border_thickness, app_rect.height() - border_thickness),
    egui::vec2(app_rect.width() - 2.0 * border_thickness, border_thickness),
  );
  let west_rect = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(0.0, border_thickness),
    egui::vec2(border_thickness, app_rect.height() - 2.0 * border_thickness),
  );
  let east_rect = egui::Rect::from_min_size(
    app_rect.min + egui::vec2(app_rect.width() - border_thickness, border_thickness),
    egui::vec2(border_thickness, app_rect.height() - 2.0 * border_thickness),
  );

  Some((north_rect, south_rect, west_rect, east_rect, top_left, top_right, bottom_left, bottom_right))
}

pub fn window_frame(
  enable: bool,
  ctx: &egui::Context,
  title: &str,
  add_contents: impl FnOnce(&mut egui::Ui),
) {
  let (is_fullscreen, is_maximized) = ctx.input(|i| {
    let viewport = i.viewport();
    (
      viewport.fullscreen.unwrap_or(false),
      viewport.maximized.unwrap_or(false),
    )
  });

  cfg_if! {
    if #[cfg(target_os = "macos")] {
      let hide = is_fullscreen;
    } else {
      let hide = false;
    }
  }

  if enable && !hide {
    let mut stroke = ctx.style().visuals.widgets.noninteractive.fg_stroke;

    let (rounding, stroke_width) = if is_fullscreen || is_maximized {
      (0.0.into(), 0.0)
    } else if HAS_BEEN_FULLSCREEN.load(Ordering::Relaxed) {
      (0.0.into(), 0.5)
    } else {
      (WINDOW_ROUNDING.into(), 0.5)
    };

    stroke.width = stroke_width;
    stroke.color = theme_color().separator_color;

    let panel_frame = egui::Frame {
      fill: theme_color().fg_color,
      rounding,
      stroke,
      outer_margin: stroke_width.into(),
      inner_margin: 0.0.into(),
      ..Default::default()
    };

    let outline_frame = egui::Frame {
      rounding,
      stroke,
      outer_margin: stroke_width.into(),
      ..Default::default()
    };

    style_setup(ctx);

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
      let app_rect = ui.max_rect();

      let title_bar_height = 28.0;
      let title_bar_rect = {
        let mut rect = app_rect;
        rect.max.y = rect.min.y + title_bar_height;
        rect
      };
      title_bar_ui(ui, title_bar_rect, title, is_fullscreen, is_maximized);

      // Add the contents:
      let content_rect = {
        let mut rect = app_rect;
        rect.min.y = title_bar_rect.max.y;
        rect
      };

      let mut content_ui = ui.child_ui(content_rect, *ui.layout(), None);
      add_contents(&mut content_ui);

      ui.painter().add(outline_frame.paint(app_rect));

      let border_thickness = if WIN32 {
        6.0
      } else {
        0.0
      };

      if !is_fullscreen {
        handle_custom_resize(ui, vec2(640.0, 480.0), app_rect, border_thickness);
      }
    });
  } else {
    let panel_frame = egui::Frame {
      fill: theme_color().fg_color,
      inner_margin: 0.0.into(),
      outer_margin: 0.0.into(),
      ..Default::default()
    };

    CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
      let app_rect = ui.max_rect();
      let mut content_ui = ui.child_ui(app_rect, *ui.layout(), None);
      add_contents(&mut content_ui);
    });
  }
}

fn title_bar_ui(
  ui: &mut egui::Ui,
  title_bar_rect: eframe::epaint::Rect,
  title: &str,
  is_fullscreen: bool,
  is_maximized: bool,
) {
  use egui::*;

  let painter = ui.painter();

  let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

  // Paint the title:
  painter.text(
    title_bar_rect.min + vec2(10.0, 5.0),
    Align2::LEFT_TOP,
    title,
    FontId::proportional(16.0),
    ui.style().visuals.text_color(),
  );

  // Paint the line under the title:
  painter.line_segment(
    [
      title_bar_rect.left_bottom() + vec2(1.0, 0.0),
      title_bar_rect.right_bottom() + vec2(-1.0, 0.0),
    ],
    ui.visuals().widgets.noninteractive.bg_stroke,
  );

  // Interact with the title bar (drag to move window):
  if title_bar_response.double_clicked() {
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    ui.ctx()
      .send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
  } else if title_bar_response.is_pointer_button_down_on() {
    ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
  }

  ui.allocate_ui_at_rect(title_bar_rect, |ui| {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
      ui.spacing_mut().item_spacing.x = 0.0;
      ui.visuals_mut().button_frame = false;
      ui.add_space(8.0);
      close_maximize_minimize(ui, is_fullscreen, is_maximized);
    });
  });
}

/// Show some close/maximize/minimize buttons for the native window.
fn close_maximize_minimize(ui: &mut egui::Ui, is_fullscreen: bool, is_maximized: bool) {
  use egui_phosphor::light::*;

  let spacing = 8.0;
  let button_height = 16.0;

  let close_response = ui
    .add(Button::new(
      RichText::new(X.to_string()).size(button_height),
    ))
    // .add(Button::new(RichText::new("❌").size(button_height)))
    .on_hover_text(i18n("Close the window"));
  if close_response.clicked() {
    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
  }

  cfg_if! {
    if #[cfg(target_os = "macos")] {
      let support_fullscreen = true;
      let support_maximize = true;
    } else {
      let support_fullscreen = true;
      let support_maximize = true;
    }
  }

  if support_fullscreen && !is_maximized {
    ui.add_space(spacing);

    let is_fullscreen = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
    if is_fullscreen {
      let fullscreen_response = ui
        // .add(Button::new(RichText::new("🗗").size(button_height)))
        .add(Button::new(
          RichText::new(ARROWS_IN.to_string()).size(button_height),
        ))
        .on_hover_text(i18n("Exit Full Screen"));
      if fullscreen_response.clicked() {
        ui.ctx()
          .send_viewport_cmd(ViewportCommand::Fullscreen(false));
      }
    } else {
      let fullscreen_response = ui
        // .add(Button::new(RichText::new("🗗").size(button_height)))
        .add(Button::new(
          RichText::new(ARROWS_OUT.to_string()).size(button_height),
        ))
        .on_hover_text(i18n("Full Screen"));
      if fullscreen_response.clicked() {
        ui.ctx()
          .send_viewport_cmd(ViewportCommand::Fullscreen(true));
        if !is_fullscreen && WIN32 {
            HAS_BEEN_FULLSCREEN.store(true, Ordering::Relaxed);
        }
      }
    }
  }

  if support_maximize && !is_fullscreen {
    ui.add_space(spacing);

    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
      let maximized_response = ui
        // .add(Button::new(RichText::new("🗗").size(button_height)))
        .add(Button::new(
          RichText::new(RECTANGLE.to_string()).size(button_height),
        ))
        .on_hover_text(i18n("Restore window"));
      if maximized_response.clicked() {
        ui.ctx()
          .send_viewport_cmd(ViewportCommand::Maximized(false));
      }
    } else {
      let maximized_response = ui
        // .add(Button::new(RichText::new("🗗").size(button_height)))
        .add(Button::new(
          RichText::new(SQUARE.to_string()).size(button_height),
        ))
        // .add(Button::new(RichText::new(ARROWS_OUT.to_string()).size(button_height)))
        .on_hover_text(i18n("Maximize window"));
      if maximized_response.clicked() {
        ui.ctx().send_viewport_cmd(ViewportCommand::Maximized(true));
      }
    }
  }
  ui.add_space(spacing + 2.0);

  let minimized_response = ui
    .add(Button::new(RichText::new("🗕").size(button_height)))
    // .add(Button::new(RichText::new(ARROW_SQUARE_DOWN.to_string()).size(button_height)))
    // .add(Button::new(RichText::new(ARROW_LINE_DOWN.to_string()).size(button_height)))
    .on_hover_text(i18n("Minimize the window"));
  if minimized_response.clicked() {
    ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true));
  }
}
