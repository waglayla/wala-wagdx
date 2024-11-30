use super::*;
use crate::components::*;

pub struct Outline {
    selected_tab: Tab,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum Tab {
    // Test,
    Wallet,
    NetworkInfo,
    WalaNode,
    WalaBridge,
    About,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            // Tab::Test => i18n("Hello!"),
            Tab::Wallet => i18n("Wallet"),
            Tab::NetworkInfo => i18n("Network Info"),
            Tab::WalaNode => i18n("WALA Node"),
            Tab::WalaBridge => i18n("WALA Bridge"),
            Tab::About => i18n("About"),
        }
    }

    fn component(&self) -> TypeId {
        match self {
            // Tab::Test => TypeId::of::<hello::Hello>(),
            Tab::Wallet => TypeId::of::<wallet_ui::OpenWallet>(),
            Tab::NetworkInfo => TypeId::of::<blank::Blank>(),
            Tab::WalaNode => TypeId::of::<console::DaemonConsole>(),
            Tab::WalaBridge => TypeId::of::<blank::Blank>(),
            Tab::About => TypeId::of::<blank::Blank>(),
        }
    }
}

impl Default for Outline {
    fn default() -> Self {
        Self {
            selected_tab: Tab::iter().next().unwrap(),
        }
    }
}

impl ComponentT for Outline {
    fn name(&self) -> Option<&'static str> {
        Some("Outline")
    }

    fn render(
        &mut self,
        core: &mut Core,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        let panel_fill = ctx.style().visuals.panel_fill;
        let darkened_fill = panel_fill.linear_multiply_rgb(0.475);

        egui::SidePanel::left("sidebar")
            .exact_width(225.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame {
                fill: darkened_fill,
                inner_margin: egui::Margin::ZERO,
                outer_margin: egui::Margin::ZERO,
                ..Default::default()
            })
            .show_inside(ui, |ui| {
                ui.set_min_height(ui.available_height());
                ui.spacing_mut().item_spacing = Vec2::ZERO;

                let info_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), 132.0));
                let info_rect = info_space.1;
                let painter = ui.painter_at(info_rect);

                let whole_pos = egui::Pos2 {
                  x: info_rect.min.x + 127.,
                  y: info_rect.max.y + 4.,
                };

                let part_pos = egui::Pos2 {
                  x: info_rect.min.x + 127.5,
                  y: info_rect.max.y - 14.8,
                };

                let sym_pos = egui::Pos2 {
                  x: info_rect.max.x - 12.5,
                  y: info_rect.max.y - 18.,
                };

                painter.text(
                  whole_pos,
                  egui::Align2::RIGHT_BOTTOM,
                  "420",
                  egui::FontId::new(92.0, get_font_family("DINishCondensed", true, false)),
                  egui::Color32::WHITE,
                );

                painter.text(
                  part_pos,
                  egui::Align2::LEFT_BOTTOM,
                  ".69M",
                  egui::FontId::new(25.0, get_font_family("DINishCondensed", true, false)),
                  egui::Color32::WHITE,
                );

                painter.text(
                  sym_pos,
                  egui::Align2::RIGHT_BOTTOM,
                  "WALA",
                  egui::FontId::new(16.0, get_font_family("DINish", false, false)),
                  theme_color().text_off_color_1,
                );

                // Set the text style for buttons
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(30.0, get_font_family("DINishCondensed", false, false))
                );

                for tab in self.available_tabs(core) {
                    if self.tab_button(ui, ctx, tab) {
                        self.selected_tab = tab;
                        // You might want to notify the core about tab changes
                        core.set_active_component_by_type(tab.component().clone());
                    }
                }
            });
    }
}

impl Outline {
  fn tab_button(&self, ui: &mut Ui, ctx: &Context, tab: Tab) -> bool {
      let panel_fill = ctx.style().visuals.panel_fill;
      let selected = self.selected_tab == tab;
      
      let mut visuals = ui.style_mut().visuals.clone();
      let bg_color = if selected {
          panel_fill
      } else {
          Color32::TRANSPARENT
      };

      visuals.widgets.inactive.weak_bg_fill = bg_color;
      visuals.widgets.active.weak_bg_fill = bg_color;
      ui.style_mut().visuals = visuals;

      let button_size = vec2(ui.available_width(), 55.0);
      let (rect, mut response) = ui.allocate_exact_size(button_size, egui::Sense::click());
      if !selected {          
          response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
      }

      let color = ctx.animate_color_with_time(
          response.id.with("outline_active_color"),
          bg_color,
          0.075
      );

      if ui.is_rect_visible(rect) {
          ui.spacing_mut().item_spacing = Vec2::ZERO;
          
          ui.painter().rect_filled(
              rect,
              Rounding::ZERO,
              color,
          );

          let size_factor = ctx.animate_value_with_time(
              response.id.with("text_size"),
              if response.hovered() { 1.05 } else { 1.0 },
              0.075,
          );

          let text_padding = 12.0;
          let text_rect = rect.shrink2(vec2(text_padding, 0.0));
          
          let text_color = if selected {
              theme_color().text_on_color_1
          } else if response.hovered() {
              theme_color().text_off_color_1.linear_multiply_rgb(1.8)
          } else {
              theme_color().text_off_color_1
          };

          let mut font_id = ui.style().text_styles[&egui::TextStyle::Button].clone();
          font_id.size *= size_factor;

          ui.painter().text(
              text_rect.left_top(),
              egui::Align2::LEFT_TOP,
              tab.label(),
              font_id,
              text_color,
          );
      }

      response.clicked()
  }

  fn available_tabs(&self, core: &Core) -> Vec<Tab> {
      let mut tabs = vec![Tab::Wallet, Tab::NetworkInfo];

      if core.settings.node.node_kind == WaglayladNodeKind::IntegratedAsDaemon {
          tabs.push(Tab::WalaNode);
          if core.settings.node.enable_bridge {
            tabs.push(Tab::WalaBridge);
          }
      }

      tabs.push(Tab::About);
      tabs
  }
}