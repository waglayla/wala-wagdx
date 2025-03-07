use crate::imports::*;
use core::cmp::max;
use std::collections::hash_map::Entry;
use xxhash_rust::xxh3::xxh3_64;

pub struct RequestUri {
  pub address : String,
  pub amount_sompi : Option<u64>,
  pub label : Option<String>,
}

impl std::fmt::Display for RequestUri {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut uri = self.address.clone();
    if let Some(amount_sompi) = self.amount_sompi {
      uri = format!("{}?amount={}", uri, sompi_to_waglayla(amount_sompi));
    }
    if let Some(label) = self.label.as_ref() {
      uri = format!("{}?label={}", uri, label);
    }
    write!(f, "{}", uri)
  }
}

#[derive(Clone, Default)]
pub struct WalletRequest {
  qr : HashMap<String, (String,load::Bytes)>,
  amount : String,
  amount_sompi : Option<u64>,
  label : String,
  error : Option<String>,
}

const EDIT_SIZE: f32 = 20.0;

impl WalletRequest {
  pub fn new() -> Self {
    Self { 
      qr : Default::default(),
      amount : String::default(),
      amount_sompi : None,
      label : String::default(),
      error : None,
    }
  }

  pub fn reset(&mut self) {
    if !self.qr.is_empty() {
      self.qr.clear();
    }
  }

  pub fn render(
    &mut self,
    open: &mut bool,
    core: &mut Core, 
    ctx: &egui::Context, 
    ui: &mut egui::Ui,
    dest: String,
  ) {
    let mut F = create_custom_popup(ctx);
    F.rounding = 10.0.into();

    let h_scale = max(450, (ui.available_width() / 1.5) as i32);
    let v_scale = max(450, (ui.available_height() / 1.33) as i32);

    let screen_rect = ctx.screen_rect();
    let default_pos = egui::Pos2 {
        x: screen_rect.center().x - (h_scale as f32 / 2.0),
        y: screen_rect.center().y - (v_scale as f32 / 2.0),
    };

    egui::Window::new(i18n("WALA Payment Request"))
      .open(open)
      .collapsible(true)
      .resizable(false)
      .default_pos(default_pos)
      .max_size([h_scale as f32, v_scale as f32])
      .frame(F)
      .show(ui.ctx(), |ui| 
      {
        egui::Frame::none()
        .inner_margin(12.0)
        .show(ui, |ui| 
        {
          ui.vertical_centered(|ui| {
            let available_rect = ui.available_rect_before_wrap();

            ui.add_space(36.0);
            ui.painter().text(
              pos2(
                available_rect.center().x,
                available_rect.min.y - 12.0,
              ),
              egui::Align2::CENTER_TOP,
              dest,
              FontId::new(33.0, get_font_family("DINishCondensed", false, false)),
              theme_color().strong_color,
            );
            ui.label("");

            // --
  
            let qr_size = 200.0;
            let rect_center = available_rect.center();
            let rect_min = rect_center - egui::vec2(qr_size / 2.0, qr_size / 2.0);
            let rect = egui::Rect::from_min_size(rect_min, egui::vec2(qr_size, qr_size));
  
            let qr_background_rect = ui.allocate_exact_size(
              rect.size(),
              egui::Sense::hover(),
            ).1;
        
            ui.painter().rect_filled(
              qr_background_rect.rect,
              egui::Rounding::same(6.0),
              egui::Color32::WHITE,
            );
  
            let request_uri = self.create_request_uri(core);
            let (qr_uri, qr_bytes) = self.qr(request_uri.to_string().as_str());
  
            let quiet_zone = 6.25;
            let qr_size_with_quiet_zone = qr_size - (quiet_zone * 2.0);
            let qr_rect = egui::Rect::from_min_size(
              qr_background_rect.rect.min + egui::vec2(quiet_zone, quiet_zone),
              egui::vec2(qr_size_with_quiet_zone, qr_size_with_quiet_zone)
            );
  
            Qr::render(ui, &qr_bytes, request_uri.address.as_str(), qr_rect);
            ui.add_space(12.0);

            // --

            ui.heading(i18n("Amount:"));
                
            let amount = self.amount.clone();
            ui.add_sized(
              [200.0, 35.0],
              TextEdit::singleline(&mut self.amount)
                .hint_text(i18n("Enter WALA here"))
                .font(FontId::proportional(EDIT_SIZE))
                .vertical_align(Align::Center),
            );

            if amount != self.amount {         
              match validate_waglayla_input(self.amount.as_str()) {
                Ok(_) => match try_waglayla_str_to_sompi(self.amount.as_str()) {
                  Ok(Some(amount_sompi)) => {
                    self.amount_sompi = Some(amount_sompi);
                    self.error = None;
                  },
                  Ok(None) => {
                    self.amount_sompi = None;
                    self.error = None;
                  },
                  Err(_err) => {
                    self.amount_sompi = None;
                    self.error = Some(i18n("Please enter a valid about of WALA").to_string());
                  },
                }
                Err(err) => {
                  self.amount_sompi = None;
                  self.error = Some(err.to_string());
                }
              }
            }

            let mut enabled = true;

            if let Some(error) = self.error.as_ref() {
              if self.amount.len() > 0 {
                enabled = false;
              }
            }
            
            ui.add_space(8.0);
            if ui.dx_large_button_enabled(enabled, i18n("Copy Request URI")).clicked() {
              ui.output_mut(|o| {
                o.copied_text = request_uri.to_string();
              });
            }

            ui.add_space(4.0);
            ui.colored_label(error_color(), self.error.clone().unwrap_or("".to_string()));
          });
        });
      });
  }

  fn create_request_uri(&self, core: &mut Core) -> RequestUri {
    let address = core.clone().current_account.unwrap().root_address().unwrap().to_string();
    let label = self.label.is_not_empty().then_some(self.label.clone());
    RequestUri {
      address,
      amount_sompi: self.amount_sompi,
      label,
    }
  }

  fn qr(&mut self, request_uri : &str) -> (String,load::Bytes) {
    let hash = format!("{:x}",xxh3_64(format!("{request_uri}{}", theme_color().name).as_bytes()));
    let (qr_uri,qr_bytes) = match self.qr.entry(hash.clone()) {
      Entry::Occupied(entry) => entry.into_mut(),
      Entry::Vacant(entry) => {
        let uri = format!("bytes://{hash}.svg");
        // let bits = qrcode::types::Mode::Alphanumeric.data_bits_count(request_uri.len());
        let qr = generate_qr_code_svg(request_uri.to_string());
        entry.insert((uri, qr.unwrap().as_bytes().to_vec().into()))
      },
    };

    (qr_uri.clone(),qr_bytes.clone())
  }
}