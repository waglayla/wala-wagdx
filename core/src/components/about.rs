use crate::imports::*;
use crate::components::{Component, ComponentT};
use crate::core::Core;

use egui_extras::{StripBuilder, Size};

pub struct About {}

// TODO: clean up this file massively

struct SocialIcon {
  icon: &'static str,
  url: Option<&'static str>,
  hover: Option<&'static str>,
  variant: &'static str,
}

const PARAGRAPH_SOCIAL: &'static str = "\
  Connect with us on Discord, Twitter, Telegram, and Reddit to stay updated \
  on the latest news, events, and features. We’re more than just a meme-coin; \
  we’re a community of passionate individuals who share a love for crypto, mining \
  and dogs!";

const PARAGRAPH_INFO: &'static str = "We are a fun fork of Kaspa with three primary goals: \
  community, scalability, and longetivity. The WALA house created WagLayla for all memecoin \
  lovers; miners, investors, and traders alike! Being built on Rust, the best qualities of the \
  KAS ecosystem shine through in WagLayla including speed, stability, and security. Wag with us, \
  and help build the ultimate memecoin community.";

const SOCIAL_ICONS: &[SocialIcon] = &[
  SocialIcon {
    icon: egui_phosphor::regular::GLOBE,
    url: Some("https://waglayla.com/"),
    hover: Some("Official Website"),
    variant: "phosphor"
  },
  SocialIcon {
    icon: egui_phosphor::fill::GITHUB_LOGO,
    url: Some("https://github.com/waglayla"),
    hover: Some("Waglayla on Github"),
    variant: "phosphor-fill"
  },
  SocialIcon {
    icon: egui_phosphor::fill::CUBE_FOCUS,
    url: Some("https://explorer.waglayla.com/"),
    hover: Some("Block Explorer"),
    variant: "phosphor-fill"
  },
  SocialIcon {
    icon: egui_phosphor::regular::X_LOGO,
    url: Some("https://x.com/WagLayla"),
    hover: Some("Waglayla on X"),
    variant: "phosphor"
  },
  SocialIcon {
    icon: egui_phosphor::fill::DISCORD_LOGO,
    url: Some("https://discord.gg/a2wcdDYds4"),
    hover: Some("Official Discord"),
    variant: "phosphor-fill"
  },
  SocialIcon {
    icon: egui_phosphor::regular::TELEGRAM_LOGO,
    url: Some("https://t.me/waglayla"),
    hover: Some("Official Telegram"),
    variant: "phosphor"
  },
];

// const EXCHANGE_BUTTONS: &'static [(&'static str, &'static str)] = &[
//   ("Bitcointry", "https://bitcointry.com/en/exchange/WALA_USDT"),
// ];

impl Default for About {
  fn default() -> Self {
    Self {}
  }
}

impl About {
  fn render_header(
    &mut self,
    ctx: &egui::Context,
    ui: &mut egui::Ui
  ) {
    let logo = &Assets::get().wala_text_logo_png;
    let window_rect = ui.min_rect();
    let available_width = ui.available_width();

    let logo_width = (available_width / 1.75).min(540.0);
    let aspect_ratio = logo.size()[0] as f32 / logo.size()[1] as f32;
    let logo_height = logo_width / aspect_ratio;

    let logo_pos = pos2(
      window_rect.min.x + available_width / 2.0,
      window_rect.min.y + logo_height / 2.0
    );

    DXImage::draw(
      ui, 
      &Assets::get().wala_text_logo_png, 
      logo_width, 
      logo_pos, 
      egui::Align2::CENTER_CENTER
    );

    let icon_size = logo_height / 2.0;

    let link_rect = egui::Rect::from_center_size(
      logo_pos + vec2(0.0, logo_height),
      vec2(logo_width, icon_size),
    );

    let num_icons = SOCIAL_ICONS.len();
    let total_spacing = (link_rect.width() - (num_icons as f32 * icon_size)).max(0.0);
    let spacing = total_spacing / (num_icons + 1) as f32;

    for (i, social_icon) in SOCIAL_ICONS.iter().enumerate() {
      let icon_center = pos2(
        link_rect.min.x + spacing * (i as f32 + 1.0) + icon_size * (i as f32 + 0.5),
        link_rect.center().y,
      );

      let icon_rect = egui::Rect::from_center_size(icon_center, vec2(icon_size, icon_size));

      let response = ui.allocate_rect(icon_rect, egui::Sense::click())
        .on_hover_text(i18n(social_icon.hover.unwrap_or("")))
        .on_hover_cursor(egui::CursorIcon::PointingHand)
      ;

      if response.clicked() {
        if let Some(url) = social_icon.url {
          open::that(url).unwrap_or_else(|_| {
            println!("Failed to open URL: {url}");
          });
        }
      }

      let ( draw_size_buffer , draw_color_buffer ) = if response.hovered {
        (icon_size*1.05, theme_color().strong_color)
      } else {
        (icon_size, theme_color().default_color)
      };

      let draw_size = ctx.animate_value_with_time(
        response.id.with(social_icon.url.unwrap_or(social_icon.icon)),
        draw_size_buffer,
        0.15
      );

      let draw_color = ctx.animate_color_with_time(
        response.id.with(format!("{}_color", social_icon.url.unwrap_or(social_icon.icon))),
        draw_color_buffer,
        0.15
      );

      ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        social_icon.icon,
        egui::FontId::new(draw_size, egui::FontFamily::Name(social_icon.variant.into())),
        draw_color,
      );
    }
  }

  fn render_paragraph_with_strip(&mut self, ui: &mut egui::Ui, max_width: f32, font_size: f32, text: &str) {
    use egui::*;

    let font_id = FontId::new(font_size, FontFamily::Proportional);
    let color = ui.visuals().text_color();

    let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
      let mut layout_job = LayoutJob::default();
      layout_job.wrap.max_width = wrap_width;
      layout_job.wrap.break_anywhere = false; // Only break at spaces
      layout_job.halign = Align::LEFT;
      layout_job.append(string, 0.0, TextFormat {
        font_id: font_id.clone(),
        color,
        ..Default::default()
      });
      ui.fonts(|f| f.layout_job(layout_job))
    };

    let galley = layouter(ui, text, max_width);

    let galley_width = galley.size().x;
    let galley_height = galley.size().y;
    let frame_width = galley_width;
    let frame_height = galley_height;

    let available_width = ui.available_width();
    let x_offset = (available_width - max_width)/2.0;

    let response = ui.allocate_rect(
        Rect::from_min_size(ui.min_rect().min, vec2(1.0, frame_height)),
        Sense::hover(),
    );

    ui.painter().rect_filled(
      Rect::from_min_size(ui.min_rect().min - vec2(frame_width/2.0 + 10.0, 10.0), vec2(frame_width + 20.0, frame_height + 20.0)),
      egui::Rounding::same(6.0),
      theme_color().bg_color,
    );
    let text_pos = response.rect.min + vec2((frame_width - galley_width) / 2.0, (frame_height - galley_height) / 2.0);
    ui.painter().galley(text_pos - vec2(frame_width/2.0,0.0), galley, Color32::WHITE);
  }

  // fn dynamic_exchange_button_area(&self, ui: &mut egui::Ui, max_width: f32, buttons: &[(&str, &str)]) {
  //   let button_padding = 6.0;
  //   let button_height = 48.0;
  //   let button_width = 180.0;

  //   let buttons_per_row = ((max_width + button_padding) / (button_width + button_padding)).floor() as usize;
  //   let rows_needed = (buttons.len() as f32 / buttons_per_row as f32).ceil() as usize;

  //   let total_height = rows_needed as f32 * (button_height + button_padding) - button_padding;

  //   egui::ScrollArea::vertical().max_height(total_height)
  //     .max_height(ui.available_height())
  //     .show(ui, |ui| 
  //   {
  //     for chunk in buttons.chunks(buttons_per_row) {
  //       ui.horizontal(|ui| {
  //         let available_width = ui.available_width();
  //         let buttons_width = chunk.len() as f32 * (button_width + button_padding) + button_padding;
  //         let centering_offset = (available_width - buttons_width) / 2.0;
  //         ui.add_space(centering_offset);

  //         for (text, url) in chunk {
  //           if ui.dx_button_sized(
  //             *text,
  //             32.0,
  //             -14.0, 
  //             Default::default(), 
  //             vec2(button_width, button_height)
  //           ).clicked() {
  //             if let Err(err) = open::that(*url) {
  //               eprintln!("Failed to open URL: {}", err);
  //             }
  //           }
  //         }
  //         if chunk.len() == buttons_per_row {
  //           ui.add_space(button_padding);
  //         }
  //       });
  //       ui.add_space(button_padding);
  //     }
  //   });
  // }
}

impl ComponentT for About {
  fn name(&self) -> Option<&'static str> {
    Some("About")
  }

  fn render(
    &mut self,
    _core: &mut Core,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    let available_width = ui.available_width();
    let window_height = ui.ctx().screen_rect().height();

    // Scale the max width based on available width
    let max_width = (available_width * 0.66).clamp(600.0, 1920.0);

    // Scale the font size based on window height
    let base_font_size = 20.0;
    let font_size = (base_font_size * (window_height / 720.0)).clamp(16.0, 32.0);

    egui::Frame::none()
      .inner_margin(20.0)
      .show(ui, |ui| 
    {
      egui::ScrollArea::vertical().auto_shrink([false,true]).show(ui, |ui| {
        self.render_header(ctx, ui);

        ui.add_space(font_size);
        ui.vertical_centered(|ui| {
          self.render_paragraph_with_strip(ui, max_width, font_size, i18n(PARAGRAPH_INFO));
        });

        ui.add_space(24.0);

        let image_pos = pos2(
          ui.available_rect_before_wrap().center().x,
          ui.available_rect_before_wrap().min.y,
        );
    
        DXImage::draw(ui, &Assets::get().paw_banner, max_width, image_pos, egui::Align2::CENTER_TOP);
        ui.add_space(24.0);

        ui.vertical_centered(|ui| {
          self.render_paragraph_with_strip(ui, max_width, font_size, i18n(PARAGRAPH_SOCIAL));
        });
        // self.dynamic_exchange_button_area(ui, max_width, &EXCHANGE_BUTTONS);
      });
    });
  }
}