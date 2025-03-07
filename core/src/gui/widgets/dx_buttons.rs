use crate::imports::*;
use egui::{self, Response, Ui, Widget, Color32};

pub fn medium_button(text: impl Into<String>) -> impl Widget {
  move |ui: &mut Ui| medium_button_ui(ui, text.into())
}

pub fn large_button(text: impl Into<String>) -> impl Widget {
  move |ui: &mut Ui| large_button_ui(ui, text.into())
}

pub fn medium_button_enabled(enabled: bool, text: impl Into<String>) -> impl Widget {
  move |ui: &mut Ui| medium_button_ui_enabled(ui, enabled, text.into())
}

pub fn large_button_enabled(enabled: bool, text: impl Into<String>) -> impl Widget {
  move |ui: &mut Ui| large_button_ui_enabled(ui, enabled, text.into())
}

fn medium_button_ui(ui: &mut Ui, text: String) -> Response {
  let min_size = ui.spacing().interact_size.y * egui::vec2(1.5, 1.0);
  ui.style_mut().spacing.button_padding = egui::vec2(10.0, 4.0);

  let button = egui::Button::new(text).min_size(min_size);
  let mut resp = ui.add(button);
  resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
  resp
}

fn large_button_ui(ui: &mut Ui, text: String) -> Response {
  let min_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.2);
  ui.style_mut().spacing.button_padding = egui::vec2(16.0, 8.0);
  let font_id = egui::FontId::proportional(16.0);

  let button = egui::Button::new(egui::RichText::new(text).font(font_id)).min_size(min_size);
  let mut resp = ui.add(button);
  resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
  resp
}

fn medium_button_ui_enabled(ui: &mut Ui, enabled: bool, text: String) -> Response {
  let min_size = ui.spacing().interact_size.y * egui::vec2(1.5, 1.0);
  ui.style_mut().spacing.button_padding = egui::vec2(10.0, 4.0);

  let button = egui::Button::new(text).min_size(min_size);
  let mut resp = ui.add_enabled(enabled, button);
  resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
  resp
}

fn large_button_ui_enabled(ui: &mut Ui, enabled: bool, text: String) -> Response {
  let min_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.2);
  ui.style_mut().spacing.button_padding = egui::vec2(16.0, 8.0);
  let font_id = egui::FontId::proportional(16.0);

  let button = egui::Button::new(egui::RichText::new(text).font(font_id)).min_size(min_size);
  let mut resp = ui.add_enabled(enabled, button);
  resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
  resp
}

#[derive(Default)]
pub enum DX_Button {
  #[default]
  Standard,
  Biscuit,
}

impl DX_Button {
  pub fn text_color(&self) -> Color32 {
    match self {
      DX_Button::Standard => theme_color().default_color,
      DX_Button::Biscuit => theme_color().bg_color,
    }
  }

  pub fn bg_color(&self) -> Color32 {
    match self {
      DX_Button::Standard => theme_color().button_color,
      DX_Button::Biscuit => theme_color().fg_color,
    }
  }

  pub fn hover_color(&self) -> Color32 {
    match self {
      DX_Button::Standard => theme_color().button_color,
      DX_Button::Biscuit => theme_color().strong_color,
    }
  }

  pub fn hover_text_color(&self) -> Color32 {
    match self {
      DX_Button::Standard => theme_color().strong_color,
      DX_Button::Biscuit => theme_color().bg_color,
    }
  }
}

pub fn dx_button_ui(
  ui: &mut egui::Ui,
  text: String,
  font_size: f32,
  padding: f32,
  kind: DX_Button,
) -> egui::Response {
  // Calculate the text width
  let text_width = ui.fonts(|fonts| {
    fonts.layout_no_wrap(
      text.to_string(),
      egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
      theme_color().bg_color,
    )
    .rect
    .width()
  });

  // Define button size based on text width and padding
  let button_width = text_width + padding * 3.0;
  let button_height = font_size + padding * 2.0;

  let button_rect = ui.allocate_exact_size(
    egui::vec2(button_width, button_height),
    egui::Sense::click() | egui::Sense::focusable_noninteractive(),
  ).1.on_hover_cursor(egui::CursorIcon::PointingHand);

  // Handle hover animation for color
  let bg_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_bg_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if button_rect.hovered() {
      kind.hover_color()
    } else {
      kind.bg_color()
    },
    0.125,
  );

  let text_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_text_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if button_rect.hovered() {
      kind.hover_text_color()
    } else {
      kind.text_color()
    },
    0.125,
  );

  let view_padding = ui.ctx().animate_value_with_time(
    ui.id().with(format!("dx_button_padding_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if button_rect.hovered() && !button_rect.clicked() {
      padding * 1.25
    } else if button_rect.clicked() {
      padding * 0.33
    } else {
      padding
    },
    0.125,
  );

  let center = button_rect.rect.center();
  let adjusted_rect = egui::Rect::from_center_size(
    center,
    egui::vec2(
      button_width - padding*3.0 + view_padding*3.0, 
      button_height - padding*2.0 + view_padding*2.0
    ),
  );

  // Draw the standard button frame
  ui.painter().rect_filled(
    adjusted_rect,
    egui::Rounding::same(6.0), // Rounded corners
    bg_color, // Background color
  );

  let stroke_width = ui.ctx().animate_value_with_time(
    ui.id().with("focus_stroke_width"),
    if button_rect.has_focus() { 1.25 } else { 0.0 },
    0.1,
  );

  ui.painter().rect_stroke(
    adjusted_rect,
    egui::Rounding::same(6.0),
    egui::Stroke::new(stroke_width, theme_color().strong_color),
  );

  // Draw the text with a vertical offset
  let text_pos = egui::Pos2 {
    x: button_rect.rect.center().x - text_width / 2.0,
    y: button_rect.rect.center().y - font_size / 2.0,
  };

  ui.painter().text(
    text_pos,
    egui::Align2::LEFT_TOP,
    text,
    egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
    text_color,
  );

  button_rect
}

pub fn dx_button_enabled_ui(
  ui: &mut egui::Ui,
  enabled: bool,
  text: String,
  font_size: f32,
  padding: f32,
  kind: DX_Button,
) -> egui::Response {
  // Calculate the text width
  let text_width = ui.fonts(|fonts| {
    fonts.layout_no_wrap(
      text.to_string(),
      egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
      theme_color().bg_color,
    )
    .rect
    .width()
  });

  // Define button size based on text width and padding
  let button_width = text_width + padding * 3.0;
  let button_height = font_size + padding * 2.0;

  let sense = if enabled { egui::Sense::click() | egui::Sense::focusable_noninteractive() } else { egui::Sense::hover() };

  // Create the button and get its response
  let mut button_rect = ui.allocate_exact_size(
    egui::vec2(button_width, button_height),
    sense,
  ).1;

  if enabled {
    button_rect = button_rect.on_hover_cursor(egui::CursorIcon::PointingHand);
  }

  // Handle hover animation for color
  let bg_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_bg_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if enabled && button_rect.hovered() {
      kind.hover_color()
    } else {
      kind.bg_color()
    },
    0.125,
  );

  let text_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_text_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if enabled && button_rect.hovered() {
      kind.hover_text_color()
    } else {
      kind.text_color()
    },
    0.125,
  );

  let view_padding = ui.ctx().animate_value_with_time(
    ui.id().with(format!("dx_button_padding_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if enabled && button_rect.hovered() && !button_rect.clicked() {
      padding * 1.25
    } else if button_rect.clicked() {
      padding * 0.33
    } else {
      padding
    },
    0.125,
  );

  let center = button_rect.rect.center();
  let adjusted_rect = egui::Rect::from_center_size(
    center,
    egui::vec2(
      button_width - padding*3.0 + view_padding*3.0, 
      button_height - padding*2.0 + view_padding*2.0
    ),
  );

  ui.painter().rect_filled(
    adjusted_rect,
    egui::Rounding::same(6.0), // Rounded corners
    bg_color, // Background color
  );

  // Draw the text with a vertical offset
  let text_pos = egui::Pos2 {
    x: button_rect.rect.center().x - text_width / 2.0,
    y: button_rect.rect.center().y - font_size / 2.0,
  };

  ui.painter().text(
    text_pos,
    egui::Align2::LEFT_TOP,
    text,
    egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
    text_color,
  );

  let stroke_width = ui.ctx().animate_value_with_time(
    ui.id().with("focus_stroke_width"),
    if enabled && button_rect.has_focus() { 1.25 } else { 0.0 },
    0.1,
  );

  ui.painter().rect_stroke(
    adjusted_rect,
    egui::Rounding::same(6.0),
    egui::Stroke::new(stroke_width, theme_color().strong_color),
  );

  // Draw the disabled color
  let over_color = Color32::from_rgba_unmultiplied(
    theme_color().fg_color.r(),
    theme_color().fg_color.g(),
    theme_color().fg_color.b(),
    theme_color().disabled_a,
  );

  if !enabled {
    ui.painter().rect_filled(
      adjusted_rect,
      egui::Rounding::same(6.0), // Rounded corners
      over_color, // Background color
    );
  }

  button_rect
}

pub fn dx_button_sized_ui(
  ui: &mut egui::Ui,
  text: String,
  font_size: f32,
  kind: DX_Button,
  size: Vec2,
) -> egui::Response {
  // Calculate the text width
  let text_width = ui.fonts(|fonts| {
    fonts.layout_no_wrap(
      text.to_string(),
      egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
      theme_color().bg_color,
    )
    .rect
    .width()
  });

  // Define button size based on text width and padding
  let button_width = size.x;
  let button_height = size.y;

  
  // Create the button and get its response
  let button_rect = ui.allocate_exact_size(
    egui::vec2(button_width, button_height),
    egui::Sense::click() | egui::Sense::focusable_noninteractive(),
  ).1.on_hover_cursor(egui::CursorIcon::PointingHand);

  // Handle hover animation for color
  let bg_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_bg_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if button_rect.hovered() {
      kind.hover_color()
    } else {
      kind.bg_color()
    },
    0.125,
  );

  let text_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_text_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if button_rect.hovered() {
      kind.hover_text_color()
    } else {
      kind.text_color()
    },
    0.125,
  );

  let view_padding = ui.ctx().animate_value_with_time(
    ui.id().with(format!("dx_button_padding_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if button_rect.hovered() && !button_rect.clicked() {
      8.0
    } else if button_rect.clicked() {
      -6.0
    } else {
      0.0
    },
    0.125,
  );

  let center = button_rect.rect.center();
  let adjusted_rect = egui::Rect::from_center_size(
    center,
    egui::vec2(
      button_width + view_padding, 
      button_height + view_padding
    ),
  );

  // Draw the standard button frame
  ui.painter().rect_filled(
    adjusted_rect,
    egui::Rounding::same(6.0), // Rounded corners
    bg_color, // Background color
  );

  // Draw the text with a vertical offset
  let text_pos = egui::Pos2 {
    x: button_rect.rect.center().x - text_width / 2.0,
    y: button_rect.rect.center().y - font_size / 2.0 - font_size/12.0,
  };

  ui.painter().text(
    text_pos,
    egui::Align2::LEFT_TOP,
    text,
    egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
    text_color,
  );

  let stroke_width = ui.ctx().animate_value_with_time(
    ui.id().with("focus_stroke_width"),
    if button_rect.has_focus() { 1.25 } else { 0.0 },
    0.1,
  );

  ui.painter().rect_stroke(
    adjusted_rect,
    egui::Rounding::same(6.0),
    egui::Stroke::new(stroke_width, theme_color().strong_color),
  );

  button_rect
}

pub fn dx_button_sized_enabled_ui(
  ui: &mut egui::Ui,
  enabled: bool,
  text: String,
  font_size: f32,
  kind: DX_Button,
  size: Vec2,
) -> egui::Response {
  // Calculate the text width
  let text_width = ui.fonts(|fonts| {
    fonts.layout_no_wrap(
      text.to_string(),
      egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
      theme_color().bg_color,
    )
    .rect
    .width()
  });

  // Define button size based on text width and padding
  let button_width = size.x;
  let button_height = size.y;

  
  let sense = if enabled { egui::Sense::click() | egui::Sense::focusable_noninteractive()} else { egui::Sense::hover() };

  // Create the button and get its response
  let mut button_rect = ui.allocate_exact_size(
    egui::vec2(button_width, button_height),
    sense,
  ).1;

  if enabled {
    button_rect = button_rect.on_hover_cursor(egui::CursorIcon::PointingHand);
  }

  // Handle hover animation for color
  let bg_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_bg_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if enabled && button_rect.hovered() {
      kind.hover_color()
    } else {
      kind.bg_color()
    },
    0.125,
  );

  let text_color = ui.ctx().animate_color_with_time(
    ui.id().with(format!("dx_button_text_color_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if enabled && button_rect.hovered() {
      kind.hover_text_color()
    } else {
      kind.text_color()
    },
    0.125,
  );

  let view_padding = ui.ctx().animate_value_with_time(
    ui.id().with(format!("dx_button_padding_{}_{}_{}", text, button_rect.rect.center().x, button_rect.rect.center().y)),
    if enabled && button_rect.hovered() && !button_rect.clicked() {
      8.0
    } else if button_rect.clicked() {
      -6.0
    } else {
      0.0
    },
    0.125,
  );

  let center = button_rect.rect.center();
  let adjusted_rect = egui::Rect::from_center_size(
    center,
    egui::vec2(
      button_width + view_padding, 
      button_height + view_padding
    ),
  );

  // Draw the standard button frame
  ui.painter().rect_filled(
    adjusted_rect,
    egui::Rounding::same(6.0), // Rounded corners
    bg_color, // Background color
  );

  // Draw the text with a vertical offset
  let text_pos = egui::Pos2 {
    x: button_rect.rect.center().x - text_width / 2.0,
    y: button_rect.rect.center().y - font_size / 2.0,
  };

  ui.painter().text(
    text_pos,
    egui::Align2::LEFT_TOP,
    text,
    egui::FontId::new(font_size, get_font_family("DINishCondensed", true, false)),
    text_color,
  );

  let stroke_width = ui.ctx().animate_value_with_time(
    ui.id().with("focus_stroke_width"),
    if button_rect.has_focus() { 1.25 } else { 0.0 },
    0.1,
  );

  ui.painter().rect_stroke(
    adjusted_rect,
    egui::Rounding::same(6.0),
    egui::Stroke::new(stroke_width, theme_color().strong_color),
  );

  // Draw the disabled color
  let over_color = Color32::from_rgba_unmultiplied(
    theme_color().fg_color.r(),
    theme_color().fg_color.g(),
    theme_color().fg_color.b(),
    theme_color().disabled_a,
  );

  if !enabled {
    ui.painter().rect_filled(
      adjusted_rect,
      egui::Rounding::same(6.0), // Rounded corners
      over_color, // Background color
    );
  }

  button_rect
}

pub fn dx_button(      
  text: impl Into<String>,
  font_size: f32,
  padding: f32,
  kind: DX_Button,
) -> impl Widget {
  move |ui: &mut Ui| dx_button_ui(ui, text.into(), font_size, padding, kind)
}

pub fn dx_button_enabled(      
  enabled: bool,
  text: impl Into<String>,
  font_size: f32,
  padding: f32,
  kind: DX_Button,
) -> impl Widget {
  move |ui: &mut Ui| dx_button_enabled_ui(ui, enabled, text.into(), font_size, padding, kind)
}

pub fn dx_button_sized(      
  text: impl Into<String>,
  font_size: f32,
  kind: DX_Button,
  size: Vec2,
) -> impl Widget {
  move |ui: &mut Ui| dx_button_sized_ui(ui, text.into(), font_size, kind, size)
}

pub fn dx_button_sized_enabled(      
  enabled: bool,
  text: impl Into<String>,
  font_size: f32,
  kind: DX_Button,
  size: Vec2,
) -> impl Widget {
  move |ui: &mut Ui| dx_button_sized_enabled_ui(ui, enabled, text.into(), font_size, kind, size)
}

// Optional: Convenience methods for Ui
pub trait ButtonExt {
  fn medium_button(&mut self, text: impl Into<String>) -> Response;
  fn large_button(&mut self, text: impl Into<String>) -> Response;
  fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response;
  fn large_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response;
  
  fn dx_button(
    &mut self, 
    text: impl Into<String>,
    font_size: f32,
    padding: f32,
    kind: DX_Button,
  ) -> Response;

  fn dx_button_enabled(
    &mut self, 
    enabled: bool,
    text: impl Into<String>,
    font_size: f32,
    padding: f32,
    kind: DX_Button,
  ) -> Response;

  fn dx_button_sized(
    &mut self, 
    text: impl Into<String>,
    font_size: f32,
    kind: DX_Button,
    size: Vec2,
  ) -> Response;

  fn dx_button_sized_enabled(      
    &mut self, 
    enabled: bool,
    text: impl Into<String>,
    font_size: f32,
    kind: DX_Button,
    size: Vec2,
  ) -> Response;

  fn dx_large_button(&mut self, text: impl Into<String>) -> Response;
  fn dx_large_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response;
}

impl ButtonExt for Ui {
  fn medium_button(&mut self, text: impl Into<String>) -> Response {
    self.add(medium_button(text))
  }

  fn large_button(&mut self, text: impl Into<String>) -> Response {
    self.add(large_button(text))
  }

  fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response {
    self.add(medium_button_enabled(enabled, text))
  }

  fn large_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response {
    self.add(large_button_enabled(enabled, text))
  }

  fn dx_button(
    &mut self, 
    text: impl Into<String>,
    font_size: f32,
    padding: f32,
    kind: DX_Button,
  ) -> Response {
    self.add(dx_button(text, font_size, padding, kind))
  }

  fn dx_button_enabled(
    &mut self, 
    enabled: bool,
    text: impl Into<String>,
    font_size: f32,
    padding: f32,
    kind: DX_Button,
  ) -> Response {
    self.add(dx_button_enabled(enabled, text, font_size, padding, kind))
  }

  fn dx_button_sized(
    &mut self, 
    text: impl Into<String>,
    font_size: f32,
    kind: DX_Button,
    size: Vec2,
  ) -> Response {
    self.add(dx_button_sized(text, font_size, kind, size))
  }

  fn dx_button_sized_enabled(
    &mut self, 
    enabled: bool,
    text: impl Into<String>,
    font_size: f32,
    kind: DX_Button,
    size: Vec2,
  ) -> Response {
    self.add(dx_button_sized_enabled(enabled, text, font_size, kind, size))
  }

  fn dx_large_button(&mut self, text: impl Into<String>) -> Response {
    self.add(dx_button(text, 24.0, 6.0, Default::default()))
  }

  fn dx_large_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response {
    self.add(dx_button_enabled(enabled, text, 24.0, 6.0, Default::default()))
  }
}
