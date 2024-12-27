use crate::imports::*;
use super::*;
use core::cmp::min;
use egui_phosphor::fill::*;

#[derive(Clone, Default)]
pub enum WalletModal {
  #[default]
  Blank,
  Send,
  Request,
  Compound,
  Contacts,
}

#[derive(Clone, Default)]
pub struct WalletBiscuit {
  account_font_size: f32,
  account_text_width: f32,
  account_text: String,

  request: WalletRequest,
  send: WalletSend,
  sweep: WalletSweep,
  export: WalletExport,

  show_send: bool,
  show_request: bool,
  show_compound: bool,
  show_contacts: bool,
  show_export: bool,
}

impl WalletBiscuit {
  pub fn new() -> Self {
    Self {
      account_font_size: 58.0,
      account_text_width: 0.0,
      account_text: "".to_string(),

      ..Default::default()
    }
  }

  pub fn update_account_font_size(
    &mut self, 
    ctx: &egui::Context, 
    current_account: &Option<Account>,
    wallet_descriptor: &Option<WalletDescriptor>,
  ) {
    let mut font_size = 58.0;

    let has_account = current_account.is_some();
    if has_account {
      let max_width = 330.0;
      let mut total_width;

      let account_name = current_account.as_ref().map_or_else(
        || i18n("No Account").to_string(),
        |account| account.name_or_id(),
      );
      let wallet_name = wallet_descriptor.as_ref().map_or_else(
        || "None",
        |wallet| wallet.title.as_deref().unwrap_or_else(||i18n("NO NAME")),
      );

      self.account_text = format!(
        "{} ({}) ",
        account_name,
        wallet_name
      ).to_string();


      loop {
        let text_width = calculate_text_width(&self.account_text, ctx, font_size, get_font_family("DINishCondensed", false, false));
        let icon_width = calculate_text_width(USER_SWITCH, ctx, font_size, egui::FontFamily::Name("phosphor-fill".into()));

        total_width = text_width + icon_width;
  
        if total_width <= max_width || font_size <= 1.0 {
          self.account_text_width = text_width;
          break;
        }
  
        font_size -= 1.0;
      }
    } else {
      println!("no account found for update");
    }
    self.account_font_size = font_size;
  }

  pub fn render(&mut self, 
    core: &mut Core, 
    ctx: &egui::Context, 
    ui: &mut egui::Ui
  ) {
    
    let rect_width = 750.0;
    let rect_height = 200.0;
    let content_size = egui::vec2(rect_width, rect_height);

    egui::ScrollArea::horizontal()
      .max_width(ui.available_width())
      .show(ui, |ui| {
      let rounding = 7.0;

      let available_width = ui.available_width();
      let offset_x = if content_size.x > available_width {
        (content_size.x - available_width) / 2.0
      } else {
        0.0
      };

      let rect = ui.allocate_space(content_size).1.translate(egui::vec2(offset_x, 0.0));;
      let painter = ui.painter();
      painter.rect_filled(
        rect,
        egui::Rounding::same(rounding), // Apply rounding to all corners
        theme_color().bg_color, // Background color
      );

      let has_account = core.current_account.is_some();
      if !has_account { return; }

      self.render_balance_section(ui, core, &rect);
      self.render_pending_section(ui, core, &rect);
      self.render_utxo_section(ui, core, &rect);
      self.render_seed_section(ui, core, &rect);

      self.render_account_section(ui, core, &rect);
      self.render_base_qr(ui, core, &rect);
      self.render_base_address(ui, core, &rect);

      self.render_buttons(ui, core, &rect);
    });

    //
    if (self.show_request) {
      self.request.render(
        &mut self.show_request,
        core,
        ctx,
        ui,
        self.account_text.clone(),
      );
    } else {
      self.request.reset();
    }
    
    if self.show_send {
      self.send.render(
        &mut self.show_send,
        core,
        ctx,
        ui,
      );
    } else {
      self.send.reset();
    }

    if self.show_compound {
      self.sweep.render(
        &mut self.show_compound,
        core,
        ctx,
        ui,
      );
    } else {
      self.sweep.reset();
    }

    if self.show_export {
      self.export.render(
        &mut self.show_export,
        core,
        ctx,
        ui,
      );
    } else {
      self.export.reset();
    }
  }

  fn render_base_qr(&mut self, ui: &mut Ui, core: &Core, rect: &Rect) {
    let total_qr_size = rect.height() - 24.0;  // Size including background
    let qr_background_rect = egui::Rect::from_min_size(
      egui::pos2(rect.max.x - total_qr_size - 12.0, rect.min.y + 12.0),
      egui::vec2(total_qr_size, total_qr_size)
    );

    let painter = ui.painter();
    painter.rect_filled(
      qr_background_rect,
      egui::Rounding::same(6.0),
      egui::Color32::WHITE,
    );

    let quiet_zone = 6.25;
    let qr_size = total_qr_size - (quiet_zone * 2.0);
    let qr_rect = egui::Rect::from_min_size(
      qr_background_rect.min + egui::vec2(quiet_zone, quiet_zone),
      egui::vec2(qr_size, qr_size)
    );

    if let Some(account) = &core.current_account {
      if let (Some(qr_bytes), Some(address)) = (account.current_qr_code(), Some(account.root_address().unwrap())) {
        Qr::render(ui, &qr_bytes, address.as_str(), qr_rect);
      }
    }
  }

  fn render_base_address(&mut self, ui: &mut Ui, core: &Core, rect: &Rect) {
    let painter = ui.painter_at(*rect);
    let address_pos = egui::Pos2 {
      x: rect.max.x - 200.0,
      y: rect.min.y + 69.0,
    };    

    if let Some(account) = &core.current_account {
      let address_txt = account.root_address().unwrap();
      let address_trunc = address_to_compact(&address_txt.clone());

      let galley = ui.fonts(|f| {
        f.layout_no_wrap(
          address_trunc.clone(),
          egui::FontId::new(20.0, get_font_family("DINishCondensed", true, false)),
          theme_color().text_off_color_1,
        )
      });

      let address_rect = egui::Rect::from_min_max(
        egui::Pos2 {
          x: address_pos.x - galley.rect.width(),
          y: address_pos.y - galley.rect.height() + 5.0,
        },
        egui::Pos2 {
          x: address_pos.x,
          y: address_pos.y + 5.0,
        },
      );

      let mut response = ui.interact(
        address_rect, 
        egui::Id::new("address_trunc_area"), 
        egui::Sense::click() | egui::Sense::hover(),
      );

      let address_color = if response.hovered() {
        theme_color().strong_color
      } else {
        theme_color().text_off_color_1
      };
  
      let color = ui.ctx().animate_color_with_time(
        response.id.with("address_trunc_color"),
        address_color,
        0.125
      );  

      painter.text(
        address_pos,
        egui::Align2::RIGHT_BOTTOM,
        address_trunc.clone(),
        egui::FontId::new(16.0, get_font_family("DINishCondensed", false, false)),
        color,
      );

      if response.clicked() {
        ui.output_mut(|o| {
          o.copied_text = address_txt.clone();
        });

        // manager().notify_clipboard(i18n("Copied to clipboard"));
      }

      response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
      response = response.on_hover_text_at_pointer(i18n("Click to copy complete address"));
    }
  }

  fn render_account_section(&mut self, ui: &mut Ui, core: &mut Core, rect: &Rect) {
    let painter = ui.painter_at(*rect);
    let account_pos = egui::Pos2 {
      x: rect.min.x + 12.0,
      y: rect.min.y + 78.0,
    };

    let has_account = core.current_account.is_some();

    let account_rect = egui::Rect::from_min_max(
      account_pos + vec2(
        0.0, 
        -self.account_font_size
      ),
      account_pos + vec2(
        self.account_text_width + self.account_font_size, 
        0.0
      ),
    );

    let mut account_response = ui.interact(
      account_rect,
      egui::Id::new("account_wallet_title"),
      egui::Sense::click(),
    );

    if has_account {
      account_response = account_response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }

    let icon_color = if has_account && account_response.hovered() {
      theme_color().strong_color
    } else {
      theme_color().null_balance_color
    };

    let color = ui.ctx().animate_color_with_time(
      account_response.id.with("account_title_color"),
      icon_color,
      0.125
    );

    painter.text(
      account_pos,
      egui::Align2::LEFT_BOTTOM,
      self.account_text.clone(),
      egui::FontId::new(self.account_font_size, get_font_family("DINishCondensed", false, false)),
      theme_color().strong_color,
    );

    painter.text(
      account_pos + vec2(self.account_text_width, -8.0),
      egui::Align2::LEFT_BOTTOM,
      USER_SWITCH,
      egui::FontId::new(self.account_font_size, egui::FontFamily::Name("phosphor-fill".into())),
      color,
    );

    let start = account_pos + vec2(0.0, 0.0);
    let end = Pos2::new(rect.max.x - 200., start.y);
    let stroke = Stroke::new(0.75, theme_color().text_off_color_1);

    painter.line_segment([start, end], stroke);

    if has_account {
      account_response.clone().on_hover_text_at_pointer(i18n("Select a different wallet"));
      if account_response.clicked() {
        core.set_active_component::<wallet_ui::CreateWallet>();
      }
    }
  }

  fn render_utxo_section(&mut self, ui: &mut Ui, core: &Core, rect: &Rect) {
    let painter = ui.painter_at(*rect);
    
    let utxo_pos = egui::Pos2 {
      x: rect.min.x + 12.,
      y: rect.min.y + 88.,
    };

    let account_clone = core.current_account.clone();

    if let Some(ref account) = account_clone {
      let balance = account.balance().unwrap_or_default();
      let utxo_count = format_number(balance.mature_utxo_count as u64);
      let utxo_text = format!("UTXOs: {}", utxo_count);

      painter.text(
        utxo_pos,
        egui::Align2::LEFT_TOP,
        utxo_text,
        egui::FontId::new(24.0, get_font_family("DINishCondensed", false, false)),
        theme_color().text_off_color_1,
      );
    }
  }

  fn render_seed_section(&mut self, ui: &mut Ui, core: &Core, rect: &Rect) {
    let painter = ui.painter_at(*rect);
    let text = i18n("Show Seed");

    let seed_pos = egui::Pos2 {
      x: rect.min.x + 12.,
      y: rect.min.y + 112.,
    };

    let account_clone = core.current_account.clone();

    let galley = ui.fonts(|f| {
      f.layout_no_wrap(
        text.to_string(),
        egui::FontId::new(24.0, get_font_family("DINishCondensed", true, false)),
        theme_color().null_balance_color,
      )
    });

    let seed_rect = egui::Rect::from_min_max(
      egui::Pos2 {
        x: seed_pos.x,
        y: seed_pos.y + 5.0,
      },
      egui::Pos2 {
        x: seed_pos.x + galley.rect.width(),
        y: seed_pos.y + galley.rect.height() + 5.0,
      },
    );

    let mut response = ui.interact(
      seed_rect, 
      egui::Id::new("show_seed_area"), 
      egui::Sense::click() | egui::Sense::hover(),
    )
    .on_hover_cursor(egui::CursorIcon::PointingHand)
    .on_hover_text_at_pointer(i18n("Display the mnemonic seed phrase, for wallet preservation"))
    ;

    let seed_color = if response.hovered() {
      theme_color().strong_color
    } else {
      theme_color().null_balance_color
    };

    let color = ui.ctx().animate_color_with_time(
      response.id.with("show_seed_color"),
      seed_color,
      0.125
    );

    if let Some(ref account) = account_clone {
      painter.text(
        seed_pos,
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::new(24.0, get_font_family("DINishCondensed", false, false)),
        color,
      );
    }

    if response.clicked() {
      self.show_export = true;
    }
  }

  fn render_balance_section(&mut self, ui: &mut Ui, core: &Core, rect: &Rect) {
    let painter = ui.painter_at(*rect);
    
    let whole_pos = egui::Pos2 {
      x: rect.max.x - 368.,
      y: rect.min.y + 159.,
    };
    let part_pos = egui::Pos2 {
      x: rect.max.x - 368.25,
      y: rect.min.y + 149.5,
    };
    let sym_pos = egui::Pos2 {
      x: rect.max.x - 200.,
      y: rect.min.y + 147.,
    };

    let account_clone = core.current_account.clone();

    let mut big_str = "000".to_string();
    let mut small_str = ".000".to_string();
    let mut balance_color = theme_color().strong_color;

    if let Some(ref account) = account_clone {
      let balance = account.balance().unwrap_or_default();
      let (whole, partial) = format_balance_split_8(balance.mature);
      big_str = whole;
      small_str = partial;
    }

    let galley = ui.fonts(|f| {
      f.layout_no_wrap(
        big_str.clone(),
        egui::FontId::new(68.0, get_font_family("DINishCondensed", true, false)),
        balance_color,
      )
    });

    let balance_rect = egui::Rect::from_min_max(
      egui::Pos2 {
        x: whole_pos.x - galley.rect.width(),
        y: whole_pos.y - 68.0,
      },
      egui::Pos2 {
        x: sym_pos.x,
        y: sym_pos.y,
      },
    );

    let mut response = ui.interact(
      balance_rect, 
      egui::Id::new("view_balance_area"), 
      egui::Sense::click() | egui::Sense::hover(),
    );

    painter.text(
      whole_pos,
      egui::Align2::RIGHT_BOTTOM,
      big_str.clone(),
      egui::FontId::new(68.0, get_font_family("DINishCondensed", true, false)),
      balance_color,
    );

    painter.text(
      part_pos,
      egui::Align2::LEFT_BOTTOM,
      small_str.clone(),
      egui::FontId::new(30.0, get_font_family("DINishCondensed", true, false)),
      balance_color,
    );

    painter.text(
      sym_pos,
      egui::Align2::RIGHT_BOTTOM,
      "WALA",
      egui::FontId::new(22.0, get_font_family("DINish", false, false)),
      balance_color,
    );

    if response.clicked() {
      let balance = account_clone.unwrap().balance().unwrap_or_default();
      let (whole_raw, frac_raw) = format_balance_split_raw(balance.mature);
      ui.output_mut(|o| {
        o.copied_text = format!("{}{}", whole_raw, frac_raw);
      });
    
      // manager().notify_clipboard(i18n("Copied to clipboard"));
    }

    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    response = response.on_hover_text_at_pointer(i18n("Click to copy total balance"));
  }

  fn render_pending_section(&mut self, ui: &mut Ui, core: &Core, rect: &Rect) {
    let painter = ui.painter_at(*rect);
    
    let whole_pos = egui::Pos2 {
        x: rect.max.x - 345.,
        y: rect.max.y + 5.,
    };
    let part_pos = egui::Pos2 {
        x: rect.max.x - 345.5,
        y: rect.max.y + 1.,
    };
    let sym_pos = egui::Pos2 {
        x: rect.max.x - 200.,
        y: rect.max.y - 2.,
    };

    let account_clone = core.current_account.clone();

    let mut big_str = "000".to_string();
    let mut small_str = ".000".to_string();
    let mut balance_color = theme_color().null_balance_color;

    if let Some(ref account) = account_clone {
      let balance = account.balance().unwrap_or_default();
      let (_, whole, partial) = format_balance_with_precision(balance.pending);
      big_str = whole;
      small_str = partial;
    }

    painter.text(
      whole_pos,
      egui::Align2::RIGHT_BOTTOM,
      big_str,
      egui::FontId::new(48.0, get_font_family("DINishCondensed", true, false)),
      balance_color,
    );

    painter.text(
      part_pos,
      egui::Align2::LEFT_BOTTOM,
      small_str,
      egui::FontId::new(32.0, get_font_family("DINishCondensed", true, false)),
      balance_color,
    );

    painter.text(
      sym_pos,
      egui::Align2::RIGHT_BOTTOM,
      "(PENDING)",
      egui::FontId::new(22.0, get_font_family("DINishCondensed", false, false)),
      balance_color,
    );
  }

  pub fn render_buttons(&mut self, ui: &mut egui::Ui, core: &Core, rect: &egui::Rect) {
    let button_area = egui::Rect::from_min_size(
      egui::Pos2 {
        x: rect.min.x + 12.0,
        y: rect.max.y - 41.5,
      },
      egui::vec2(300.0, 40.0), // Adjust width and height as needed
    );

    let spacing = 2.5;

    ui.allocate_ui_at_rect(button_area, |ui| {
      ui.horizontal(|ui| {
        // Add buttons in the bottom-left corner
        if ui.dx_button(i18n("Send"), 22.0, 4.0, DX_Button::Biscuit).clicked() {
          self.show_send = true;
        }

        ui.add_space(spacing);

        if ui.dx_button(i18n("Request"), 22.0, 4.0, DX_Button::Biscuit).clicked() {
          self.show_request = true;
        }

        ui.add_space(spacing);

        if ui.dx_button(i18n("Compound"), 22.0, 4.0, DX_Button::Biscuit).clicked() {
          self.show_compound = true;
        }

        ui.add_space(spacing);

        if ui.dx_button(i18n("Contacts"), 22.0, 4.0, DX_Button::Biscuit)
          .on_hover_cursor(egui::CursorIcon::Help)
          .on_hover_text_at_pointer(i18n("Feature Coming Soon..."))
          .clicked() {

        }
      });
    });
  }
}