use crate::imports::*;
use egui_phosphor::fill::*;
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
    ) 
    {
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

          // Language selection
          self.language_select_ui(core, ui, ctx);

          ui.separator();

          // Node status
          self.node_status(core, ui, ctx);

          ui.separator();
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
          .frame(create_custom_popup(ctx))
          .default_size([400.0, 400.0])
          .show(ctx, |ui| {
            let mut binding = core.clone();
            let mut settings = binding.get_mut::<components::settings::Settings>();
            settings.render(core, ctx, _frame, ui);
          });
      }
  }
}

impl Footer {
  fn node_status(&self, core: &mut Core, ui: &mut egui::Ui, ctx: &egui::Context) {
    let footer_height = 28.0;
    let desired_width = 180.0;
    let icon_size = 24.0;

    let (status_message, status_color) = describe_sync(core.node_state());
    let (icon_text, icon_color) = connection_icon(core.node_state());

    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
      ui.add_space(4.0);

      ui.add_sized(
        [icon_size, footer_height],
        egui::Label::new(
          egui::RichText::new(icon_text)
            .font(egui::FontId::new(icon_size, egui::FontFamily::Name("phosphor-bold".into())))
            .color(icon_color),
        ),
      );

      ui.add_sized(
        [desired_width - icon_size, footer_height],
        egui::Label::new(
          egui::RichText::new(status_message).color(status_color),
        ),
      );

      ui.separator();

      ui.add_sized(
        [70.0, footer_height],
        egui::Label::new(
          egui::RichText::new(describe_peers(core.node_state())).color(theme_color().separator_color),
        ),
      );
    });
  }

  fn animated_gear_button(&self, ui: &mut egui::Ui, ctx: &egui::Context) -> egui::Response {
    let response = ui.allocate_response(
      Vec2::splat(24.0), 
      egui::Sense::click()
    );

    let gear_size = 24.0;

    // Animate color
    let default_color = ui.visuals().text_color();
    let target_color = if response.hovered() { theme_color().text_on_color_1 } else { default_color };

    let color = ctx.animate_color_with_time(
      response.id.with("gear_color"),
      target_color,
      0.1
    );

    // Layout text
    let galley = ui.fonts(|f| {
      f.layout_no_wrap(
        GEAR_SIX.to_string(),
        FontId::new(gear_size, egui::FontFamily::Name("phosphor-fill".into())),
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
    
    // Define our adjustment factors
    let pivot_adjustment = vec2(0.25, -0.5);  // Adjusts the rotation center point
    let final_offset = vec2(0.0, 0.0);       // Final position adjustment
    
    // Calculate position with explicit steps
    let base_center_offset = galley.size() / 2.0;
    let adjusted_pivot = base_center_offset + pivot_adjustment;
    
    // Apply rotation around adjusted pivot
    let rotation = Rot2::from_angle(angle);
    let rotated_pos = center - rotation * adjusted_pivot;
    
    // Apply final positioning adjustment
    let final_pos = rotated_pos - final_offset;

    let mut text = epaint::TextShape::new(
        final_pos,
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

  fn language_select_ui(&self, core: &mut Core, ui: &mut egui::Ui, ctx: &egui::Context) {
    ui.add_space(3.0);

    let dictionary = i18n::dictionary();
    let lang_menu = RichText::new(format!("{} ⏶", dictionary.current_title()));
    #[allow(clippy::useless_format)]
    ui.menu_button(lang_menu, |ui| {
      cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
          let disable = ["ar","fa","he","hi","ja","ko","zh"];
        } else {
          let disable = [];
        }
      }
  
      // Adjust the maximum height of the menu to half the screen height
      let max_height = ui.ctx().screen_rect().height() / 2.0;
  
      // Add scrollable content
      ScrollArea::vertical()
        .max_height(max_height)
        .show(ui, |ui| {
          dictionary
            .enabled_languages()
            .into_iter()
            .filter(|(code, _)| !disable.contains(&code.as_str()))
            .for_each(|(code, lang)| {
              let line_height = match code {
                "ar" | "fa" => Some(26.),
                "zh" | "ko" | "ja" => Some(20.),
                "hi" | "he" => Some(10.),
                _ => None,
              };

              let size = vec2(100., 24.);
              if ui
                .add_sized(
                  size,
                  Button::new(RichText::new(lang).line_height(line_height)),
                )
                .clicked()
              {
                core.settings.language_code = code.to_string();
                dictionary
                  .activate_language_code(code)
                  .expect("Unable to activate language");
                core.settings.language_code = code.to_string();
                core.store_settings();
                ui.close_menu();
              }
        });
      });
    });

    ui.add_space(3.0);
  }
}