use crate::imports::*;
pub const BOTTOM_SPACE: f32 = 10.;

pub fn render_centered_content_noback<F, A>(
  ctx: &egui::Context,
  ui: &mut egui::Ui,
  title: &str,
  est_height: f32,
  content: F,
) -> A
where
  F: FnOnce(&mut egui::Ui) -> A,
  A: WizardActionTrait,
{
  let mut action = None;

  egui::CentralPanel::default()
    .frame(create_custom_frame(ctx))
    .show_inside(ui, |ui| {
      // Header
      let header_height = 64.;
      let header_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), header_height));
      let header_rect = header_space.1;
      let painter = ui.painter_at(header_rect);

      painter.text(
        header_rect.center() + vec2(0.0, 8.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
        theme_color().text_on_color_1,
      );

      ui.add_space(4.);
      ui.separator();

      ui.add_space(((ui.available_height() - est_height)/2.0).max(5.0));
      egui::ScrollArea::vertical()
        .auto_shrink([false,true])
        .max_height(ui.available_height())
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
              action = Some(content(ui));
            });
        });
    });

  action.unwrap_or_else(|| panic!("Content must produce a valid WizardAction"))
}

pub fn render_centered_content<F, A>(
  ctx: &egui::Context,
  ui: &mut egui::Ui,
  title: &str,
  est_height: f32,
  content: F,
) -> A
where
  F: FnOnce(&mut egui::Ui) -> A,
  A: WizardActionTrait,
{
  let mut action = None;

  egui::CentralPanel::default()
    .frame(create_custom_frame(ctx))
    .show_inside(ui, |ui| {
      // Header
      let header_height = 64.;
      let header_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), header_height));
      let header_rect = header_space.1;
      let painter = ui.painter_at(header_rect);

      painter.text(
        header_rect.center() + vec2(0.0, 8.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
        theme_color().text_on_color_1,
      );

      ui.add_space(4.);
      ui.separator();

      let available_height = ui.available_height();
      ui.add_space(((available_height - est_height) / 2.0).max(5.0));
      egui::ScrollArea::vertical()
        .max_height(available_height - 60.)
        .show(ui, |ui| {
          let content_height = ui.available_height();

          ui.vertical_centered(|ui| {
            action = Some(content(ui));
          });
        });
      // Footer
      ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        ui.add_space(BOTTOM_SPACE);
        if ui.dx_large_button(i18n("Back")).clicked() {
          action = Some(A::from_back());
        }
        ui.add_space(4.);
        ui.separator();
        ui.add_space(4.);
      });
    });

  action.unwrap_or_else(|| panic!("Content must produce a valid WizardAction"))
}