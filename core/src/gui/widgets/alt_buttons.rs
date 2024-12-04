use egui::{self, Response, Ui, Widget};

/// Medium-sized button with standardized styling
pub fn medium_button(text: impl Into<String>) -> impl Widget {
    move |ui: &mut Ui| medium_button_ui(ui, text.into())
}

/// Large-sized button with standardized styling
pub fn large_button(text: impl Into<String>) -> impl Widget {
    move |ui: &mut Ui| large_button_ui(ui, text.into())
}

/// Medium-sized button with enabled/disabled state
pub fn medium_button_enabled(enabled: bool, text: impl Into<String>) -> impl Widget {
    move |ui: &mut Ui| medium_button_ui_enabled(ui, enabled, text.into())
}

/// Large-sized button with enabled/disabled state
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

// Optional: Convenience methods for Ui
pub trait ButtonExt {
    fn medium_button(&mut self, text: impl Into<String>) -> Response;
    fn large_button(&mut self, text: impl Into<String>) -> Response;
    fn medium_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response;
    fn large_button_enabled(&mut self, enabled: bool, text: impl Into<String>) -> Response;
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
}
