use crate::imports::*;

pub enum Confirm {
  Yes,
  No,
}

pub fn confirm_ui(
  ui: &mut egui::Ui,
  confirm: &mut Option<Confirm>,
) -> egui::Response {
  // Define the size of the widget
  let desired_size = ui.spacing().interact_size.y * egui::vec2(3.0, 2.0); // Adjust for message space
  let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

  if ui.is_rect_visible(rect) {
      ui.vertical(|ui| {
          ui.horizontal(|ui| {
              // Render "Yes" button
              if ui.medium_button(i18n("Yes")).clicked() {
                  *confirm = Some(Confirm::Yes);
              }
              // Render "No" button
              if ui.medium_button(i18n("No")).clicked() {
                  *confirm = Some(Confirm::No);
              }
          });
      });
  }

  response
}

pub fn confirm_labels_ui(
  ui: &mut egui::Ui,
  yes_label: &str,
  no_label: &str,
  confirm: &mut Option<Confirm>,
) -> egui::Response {
  // Define the size of the widget
  let desired_size = ui.spacing().interact_size.y * egui::vec2(3.0, 2.0); // Adjust for message space
  let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

  if ui.is_rect_visible(rect) {
      ui.vertical(|ui| {
          ui.horizontal(|ui| {
              // Render "Yes" button
              if ui.medium_button(yes_label).clicked() {
                  *confirm = Some(Confirm::Yes);
              }
              // Render "No" button
              if ui.medium_button(no_label).clicked() {
                  *confirm = Some(Confirm::No);
              }
          });
      });
  }

  response
}

pub trait UiExtensions {
  fn confirm_widget(
    &mut self, 
  ) -> Option<Confirm>;

  fn confirm_widget_labels(
    &mut self,
    yes_label: &str, 
    no_label: &str
  ) -> Option<Confirm>;
}

impl UiExtensions for egui::Ui {
  fn confirm_widget(&mut self) -> Option<Confirm> {
      let mut confirm = None;
      confirm_ui(self, &mut confirm);
      confirm
  }

  fn confirm_widget_labels(
    &mut self,
    yes_label: &str, 
    no_label: &str
  ) -> Option<Confirm> {
    let mut confirm = None;
    confirm_labels_ui(self, yes_label, no_label, &mut confirm);
    confirm
  }
}