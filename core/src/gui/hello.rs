use eframe::egui;

pub struct MyApp {
  name: String,
}

impl Default for MyApp {
  fn default() -> Self {
    Self {
      name: "World".to_owned(),
    }
  }
}

impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.heading(format!("Hello, {}!", self.name));
      ui.horizontal(|ui| {
        ui.label("Your name: ");
        ui.text_edit_singleline(&mut self.name);
      });
    });
  }
}

pub fn run_app() -> eframe::Result<()> {
  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_inner_size([400.0, 300.0])
      .with_min_inner_size([300.0, 220.0])
      .with_decorations(false)
      .with_transparent(true),
    ..Default::default()
  };
  
  eframe::run_native(
    "My egui App",
    native_options,
    Box::new(|_cc| Ok(Box::new(MyApp::default()))),
  )
}