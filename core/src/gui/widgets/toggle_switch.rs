//! Source code example of how to create your own widget.
//! This is meant to be read as a tutorial, hence the plethora of comments.

/// iOS-style toggle switch:
///
/// ``` text
///      _____________
///     /       /.....\
///    |       |.......|
///     \_______\_____/
/// ```
///
/// ## Example:
/// ``` ignore
/// toggle_ui(ui, &mut my_bool);
/// ```
pub fn toggle_ui(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
  let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
  let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

  if response.clicked() {
      *on = !*on;
      response.mark_changed();
  }

  response.widget_info(|| {
      egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, "")
  });

  if ui.is_rect_visible(rect) {
      let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
      let mut visuals = ui.style().interact_selectable(&response, *on);

      if *on {
          visuals.bg_fill = egui::Color32::from_rgba_premultiplied(46, 182, 52, 255);
      }

      let rect = rect.expand(visuals.expansion);
      let radius = 0.5 * rect.height();
      ui.painter()
          .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
      let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
      let center = egui::pos2(circle_x, rect.center().y);
      ui.painter()
          .circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
  }

  response
}

pub fn toggle(on: &mut bool) -> impl egui::Widget + '_ {
  move |ui: &mut egui::Ui| toggle_ui(ui, on)
}

pub fn url_to_file_source_code() -> String {
  format!("https://github.com/emilk/egui/blob/master/{}", file!())
}