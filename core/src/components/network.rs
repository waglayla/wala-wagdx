use super::*;
use crate::components::{Component, ComponentT};
use crate::core::Core;

#[derive(Default)]
pub struct NetworkInfo {}

fn large_text(text: String, pos: Vec2, align: egui::Align2, color: Color32, ui: &mut egui::Ui) {
  ui.painter().text(
    ui.available_rect_before_wrap().min + pos,
    align,
    text,
    egui::FontId::new(60.0, get_font_family("DINishCondensed", false, false)),
    color,
  );
}

fn format_amount(val: u64) -> String {
  let (_, whole, part) = format_balance_with_precision(val);
  format!("{}{}", whole, part).to_string()
}

fn mined_perc(current: u64, max: u64) -> String {
  let res = current as f64 / max as f64;
  format!("{:.2}%", res * 100.0).to_string()
}

fn show_stat(name:&str, text: String, pos: Vec2, ui: &mut egui::Ui) {
  let available_rect = ui.available_rect_before_wrap();

  let bg_rect = egui::Rect::from_min_max(
    available_rect.min + pos - vec2(95.0, 6.0),
    available_rect.min + pos + vec2(95.0, 73.0 + 6.0),
  );
  ui.painter().rect_filled(
    bg_rect,
    egui::Rounding::same(6.0),
    theme_color().button_color,
  );
  ui.painter().text(
    available_rect.min + pos,
    Align2::CENTER_TOP,
    name,
    egui::FontId::new(26.0, get_font_family("DINishCondensed", false, false)),
    theme_color().default_color,
  );
  ui.painter().text(
    available_rect.min + pos + vec2(0.0, 25.0),
    Align2::CENTER_TOP,
    text,
    egui::FontId::new(46.0, get_font_family("DINishCondensed", true, false)),
    theme_color().strong_color,
  );
}

impl ComponentT for NetworkInfo {
  fn name(&self) -> Option<&'static str> {
    Some("Network Info")
  }

  fn render(
    &mut self,
    core: &mut Core,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    DXImage::paint_at(
      ui, 
      &Assets::get().paw_watermark,
      ui.available_width().min(ui.available_height()),
      ui.available_rect_before_wrap().center(), Align2::CENTER_CENTER
    );

    let mut endpoint = core.node_state().url().clone().unwrap_or("N/A".to_string());
    endpoint = if endpoint.contains("127.0.0.1") {
      "localhost".to_string() 
    } else {
      endpoint
    };

    let x_area = ui.available_width();

    large_text(i18n("Metrics - Mainnet").to_string(), vec2(x_area / 2.0, 10.0), Align2::CENTER_TOP, theme_color().default_color, ui);
    ui.painter().text(
      ui.available_rect_before_wrap().min + vec2(x_area / 2.0, 75.0),
      Align2::CENTER_TOP,
      format!("@{}", endpoint),
      egui::FontId::new(20.0, get_font_family("DINishCondensed", false, false)),
      theme_color().default_color,
    );
    ui.add_space(120.0);

    let x_0 = x_area / 4.0; 
    let x_1 = x_area / 2.0; 
    let x_2 = x_area - x_area / 4.0; 

    let current_supply = core.node_state().current_supply().unwrap_or(0_u64);
    let max_supply = core.node_state().max_supply().unwrap_or(0_u64);

    ui.horizontal(|ui| {
      show_stat(
        i18n("Current DAA"), 
        format_number(core.node_state().current_daa_score().unwrap_or(0_u64)), 
        vec2(x_0, 10.0), 
        ui
      );
      show_stat(
        i18n("Mempool Size"), 
        format!("{}", core.node_state().mempool_size().unwrap_or(0_usize)).to_string(), 
        vec2(x_1, 10.0),
        ui
      );
      show_stat(
        i18n("Block Reward"), 
        format_amount(core.node_state().block_reward().unwrap_or(0)).to_string(), 
        vec2(x_2, 10.0), 
        ui
      );
    });

    ui.add_space(100.0);
    ui.horizontal(|ui| {
      show_stat(
        i18n("Current Supply"), 
        format_amount(current_supply).to_string(), 
        vec2(x_0, 10.0), 
        ui
      );
      show_stat(
        i18n("Net. Hashrate"), 
        format_hashrate(core.node_state().hashes_per_second().unwrap_or(0)), 
        vec2(x_1, 10.0), 
        ui
      );
      show_stat(
        i18n("Difficulty"), 
        format_diff(core.node_state().difficulty().unwrap_or(0)), 
        vec2(x_2, 10.0), 
        ui
      );
    });

    ui.add_space(100.0);
    ui.horizontal(|ui| {
      show_stat(
        i18n("Max Supply"), 
        format_amount(max_supply).to_string(), 
        vec2(x_0, 10.0), 
        ui
      );
      show_stat(
        i18n("Mined Supply"), 
        mined_perc(
          current_supply,
          max_supply,
        ), 
        vec2(x_1, 10.0), 
        ui
      );
      show_stat(
        i18n("Node Version"), 
        core.node_state().server_version().clone().unwrap_or("unknown".to_string()), 
        vec2(x_2, 10.0), 
        ui
      );
    });
  }
}