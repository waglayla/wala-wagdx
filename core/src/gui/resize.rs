use crate::imports::*;
use super::*;

fn calculate_resizable_borders(
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

pub fn handle_custom_resize(
  ui: &mut egui::Ui, 
  min_size: egui::Vec2, 
  app_rect: egui::Rect,
  border_thickness: f32
) {
  if border_thickness <= 0.0 {
    return;
  }

  // Get the resizable borders
  if let Some((
    N, S, W, E,
    NW, NE, SW, SE
  )) =
      calculate_resizable_borders(app_rect, border_thickness)
  {
    egui::ViewportCommand::MinInnerSize(min_size);

    // Resize on interaction
    let drag_w = ui.interact(W, egui::Id::new("resize_west"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);

    let drag_n = ui.interact(N, egui::Id::new("resize_north"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeVertical);

    let drag_s = ui.interact(S, egui::Id::new("resize_south"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeVertical);

    let drag_e = ui.interact(E, egui::Id::new("resize_east"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeHorizontal);

    // --

    let drag_nw = ui.interact(NW, egui::Id::new("resize_north_west"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeNwSe);

    let drag_ne = ui.interact(NE, egui::Id::new("resize_north_east"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeNeSw);

    let drag_sw = ui.interact(SW, egui::Id::new("resize_south_west"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeNeSw);

    let drag_se = ui.interact(SE, egui::Id::new("resize_south_east"), egui::Sense::click_and_drag())
      .on_hover_cursor(egui::CursorIcon::ResizeNwSe);

    // Cardinal

    if drag_w.dragged() {
      let current_pos = drag_w.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::West));
      }
    }

    if drag_n.dragged() {
      let current_pos = drag_n.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::North));
      }
    }

    if drag_s.dragged() {
      let current_pos = drag_s.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::South));
      }
    }

    if drag_e.dragged() {
      let current_pos = drag_e.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::East));
      }
    }

    // Corner

    if drag_nw.dragged() {
      let current_pos = drag_nw.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::NorthWest));
      }
    }

    if drag_ne.dragged() {
      let current_pos = drag_ne.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::NorthEast));
      }
    }


    if drag_sw.dragged() {
      let current_pos = drag_sw.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthWest));
      }
    }

    if drag_se.dragged() {
      let current_pos = drag_se.interact_pointer_pos();
      if let Some(pos) = current_pos {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthEast));
      }
    }

    // Visual feedback for the borders (debug)
    // let painter = ui.painter();
    // painter.rect_stroke(N, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
    // painter.rect_stroke(S, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
    // painter.rect_stroke(W, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
    // painter.rect_stroke(E, 0.0, (1.0, egui::Color32::LIGHT_BLUE));

    // painter.rect_stroke(NW, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
    // painter.rect_stroke(NE, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
    // painter.rect_stroke(SW, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
    // painter.rect_stroke(SE, 0.0, (1.0, egui::Color32::LIGHT_BLUE));
  }
}