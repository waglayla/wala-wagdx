use crate::imports::*;
use std::collections::VecDeque;

pub struct DaemonConsole {
    lines: VecDeque<String>,
    max_lines: usize,
    receiver: Receiver<DaemonMessage>,
    font_size: f32,
}

impl DaemonConsole {
    pub fn new(receiver: Receiver<DaemonMessage>) -> Self {
        Self {
            lines: VecDeque::new(),
            max_lines: 1000,
            receiver,
            font_size: 12.0
        }
    }

    pub fn add_line(&mut self, line: String) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn update(&mut self) {
        while let Ok(DaemonMessage(line)) = self.receiver.try_recv() {
            self.add_line(line);
        }
    }

    fn get_color_for_line(line: &str) -> egui::Color32 {
        if line.contains("[ERROR]") {
            egui::Color32::RED
        } else if line.contains("[WARN]") {
            egui::Color32::YELLOW
        } else if line.contains("[INFO]") {
            egui::Color32::WHITE
        } else {
            egui::Color32::GRAY
        }
    }
}

impl ComponentT for DaemonConsole {
  fn name(&self) -> Option<&'static str> {
      Some("Waglaylad Console")
  }

  fn render(
      &mut self,
      _core: &mut Core,
      ctx: &egui::Context,
      _frame: &mut eframe::Frame,
      ui: &mut egui::Ui,
  ) {
      self.update();
      
      egui::CollapsingHeader::new("Waglaylad Console")
          .default_open(true)
          .show(ui, |ui| {
              let available_height = ui.available_height();
              ui.horizontal(|ui| {
                  if ui.button("Clear").clicked() {
                      self.lines.clear();
                  }
                  // if ui.button("Export").clicked() {
                  //     // Implement export functionality
                  // }
                  ui.label("Font size:");
                  ui.add(egui::Slider::new(&mut self.font_size, 8.0..=20.0));
              });

              let frame_height = available_height - ui.spacing().interact_size.y - 20.0;

              egui::Frame::none()
                  .fill(egui::Color32::BLACK)
                  .rounding(egui::Rounding::same(5.0))
                  .inner_margin(3.0)
                  .outer_margin(3.0)
                  .show(ui, |ui| {
                      egui::ScrollArea::vertical()
                          .max_height(frame_height)
                          .stick_to_bottom(true)
                          .show(ui, |ui| {
                              for line in &self.lines {
                                  ui.colored_label(
                                    Self::get_color_for_line(line), 
                                    egui::RichText::new(line).size(self.font_size)
                                  );
                              }
                          });
                  });
          });
  }
}