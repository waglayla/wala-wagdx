use super::*;
use crate::components::{Component, ComponentT};
use crate::core::Core;

#[derive(Default)]
pub struct Donate {}

const DEVOPS_STR: &str = "Any funds donated to this address will be used to compensate the creators \
  of the project and help cover operational expenses.";
const MARKETING_STR: &str = "Any funds donated to this address will be used to cover marketing expenses \
  of the project.";
const EXCHANGE_STR: &str = "Any funds donated to this address will be used to cover listing fees and \
  liquidity pools.";

struct Cookie {
  title: &'static str,
  description: &'static str,
  address: &'static str,
}

const COOKIES: [Cookie; 3] = [
  Cookie {
    title: "Dev/Ops Fund",
    description: DEVOPS_STR,
    address: "waglayla:qqnz8s8xcrvjykdq326umlaz0xnp49wf3gxnun5rcp2xjzfux9p6sg2acf7jd",
  },
  Cookie {
    title: "Marketing Fund",
    description: MARKETING_STR,
    address: "waglayla:qqhuw3n047uld50u0q83xg8rxm4ypnv3s0syve068l52hmtzlwlskcqdwzw05",
  },
  Cookie {
    title: "Exchange Fund",
    description: EXCHANGE_STR,
    address: "waglayla:qq7fhfdzr9280jtv42eqtn7avcxlg2rzprq6pvznyny97lv4lqh6gzz3rc2mz",
  },
];

impl ComponentT for Donate {
  fn name(&self) -> Option<&'static str> {
    Some("Donate")
  }

  fn render(
    &mut self,
    core: &mut Core,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    ui.set_height(ui.available_height());
    let base_estimated_height = 500.0;
    let font_size = (ui.available_height() / 28.5).min(ui.available_width() / 28.5);
    let factor = font_size/19.5;

    render_centered_content_noback(ctx, ui, i18n("Donation Wallets"), base_estimated_height*factor, |ui| {
      DXImage::paint_at(
        ui, 
        theme_accent_img(),
        ui.available_width().min(ui.available_height()),
        ui.available_rect_before_wrap().center() - vec2(ui.available_width()/2.0, 0.0), 
        Align2::CENTER_CENTER
      );

      DXImage::paint_at(
        ui, 
        theme_accent_img(),
        ui.available_width().min(ui.available_height()),
        ui.available_rect_before_wrap().center() + vec2(ui.available_width()/2.0, 0.0), 
        Align2::CENTER_CENTER
      );

      for cookie in COOKIES {
        ui.allocate_ui_with_layout(
          egui::vec2(540.0*factor, egui::Ui::available_height(ui)),
          egui::Layout::top_down(egui::Align::Min),
          |ui| {
            egui::Frame::none()
              .fill(theme_color().bg_color)
              .rounding(egui::Rounding::same(10.0))
              .inner_margin(10.0)
              .show(ui, |ui| {
                ui.horizontal(|ui| {
                  ui.add_space(345.0*factor);
                  let copy_size = (font_size * 1.8).max(18.0);
                  egui::Frame::none()
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                      if ui.dx_button_sized(i18n("Copy Address"), copy_size, DX_Button::Biscuit, vec2(180.0*factor, 58.0*factor)).clicked() {
                        ui.output_mut(|o| {
                          o.copied_text = cookie.address.to_string();
                        });
                        core.notify_copy();
                      }
                    });
                });
                ui.painter().text(
                  ui.min_rect().min,
                  egui::Align2::LEFT_TOP,
                  i18n(cookie.title),
                  egui::FontId::new(52.0*factor, get_font_family("DINishCondensed", true, false)),
                  theme_color().strong_color,
                );
                ui.label(
                  egui::RichText::new(i18n(cookie.description))
                    .font(egui::FontId::new(font_size, get_font_family("DINishCondensed", false, false)))
                    .color(theme_color().default_color)
                );
              });
          }
        );
        ui.add_space(16.0);
      }
      WizardAction::NoAction
    });
  }
}
