use crate::imports::*;
use egui_phosphor::light::*;
use crate::components::emath::Rot2;
pub use crate::gui;

#[derive(Default)]
pub struct Footer {
    show_settings: bool,
}

impl ComponentT for Footer {
    fn name(&self) -> Option<&'static str> {
        Some("Footer")
    }

    fn render(
        &mut self,
        core: &mut Core,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let footer_height = 28.0;
        let footer_rect = ui.max_rect().shrink2(egui::vec2(0.0, ui.max_rect().height() - footer_height));

        ui.allocate_ui_at_rect(footer_rect, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.visuals_mut().button_frame = false;

                ui.add_space(3.0);

                // Settings gear icon
                let gear_response = self.animated_gear_button(ui, ctx);
                
                if gear_response.clicked() {
                    self.show_settings = true;
                }

                ui.separator();

                // Add more footer items here as needed
            });
        });

        // Draw a line above the footer
        let stroke = ctx.style().visuals.widgets.noninteractive.bg_stroke;
        ui.painter().line_segment(
            [footer_rect.left_top(), footer_rect.right_top()],
            stroke,
        );

        // Render settings window if show_settings is true
        if self.show_settings {
            egui::Window::new("Settings")
                .open(&mut self.show_settings)
                .default_size([400.0, 300.0])
                .show(ctx, |ui| {
                    let mut binding = core.clone();
                    let mut settings = binding.get_mut::<components::settings::Settings>();
                    settings.render(core, ctx, _frame, ui);
                });
        }
    }
}

impl Footer {
  fn animated_gear_button(&self, ui: &mut egui::Ui, ctx: &egui::Context) -> egui::Response {
      let response = ui.allocate_response(
          Vec2::splat(24.0), 
          egui::Sense::click()
      );

      let gear_size = 24.0;

      // Animate color
      let default_color = ui.visuals().text_color();
      let target_color = if response.hovered() { Color32::WHITE } else { default_color };

      let color = ctx.animate_color_with_time(
          response.id.with("gear_color"),
          target_color,
          0.1
      );

      // Layout text
      let galley = ui.fonts(|f| {
          f.layout_no_wrap(
              GEAR_SIX.to_string(),
              FontId::proportional(gear_size),
              color,
          )
      });

      // Animate rotation
      let target_angle = if response.hovered() { 45.0_f32.to_radians() } else { 0.0 };
      let angle = ctx.animate_value_with_time(
          response.id.with("gear_rotation"),
          target_angle,
          0.2
      );

      let center = response.rect.center();
      
      // Use rotation matrix to correctly position the text
      let rotation = Rot2::from_angle(angle);
      let pos = center - rotation * (galley.size() / 2.0);

      let mut text = epaint::TextShape::new(
          pos,
          galley,
          color,
      );
      text.angle = angle;

      ui.painter().add(epaint::Shape::Text(text));

      // Request repaint while animating
      if response.hovered() || angle != 0.0 {
          ctx.request_repaint();
      }

      response
          .on_hover_text(i18n("Settings"))
          .on_hover_cursor(egui::CursorIcon::PointingHand)
  }
}