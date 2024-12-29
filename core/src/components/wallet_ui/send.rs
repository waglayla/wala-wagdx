use crate::imports::*;
use waglayla_wallet_core::tx::{GeneratorSummary, PaymentOutput, PaymentDestination, Fees};
use waglayla_wallet_core::api::{AccountsSendRequest, AccountsSendResponse};
use egui_extras::RetainedImage;
use std::sync::{Arc, Mutex};
use core::cmp::max;

#[derive(Default, Clone, PartialEq)]
enum SendState {
  #[default]
  Details,
  Confirm,
  Success,
}

define_indexed_enum!(
  Focus,
  Address,
  Amount,
  WalletSecret,
  PaymentSecret
);

#[derive(Clone, Default)]
pub struct WalletSend {
  pub address: String,
  pub amount: String,
  pub wallet_secret: String,
  pub payment_secret: String,
  pub amount_sompi : Option<u64>,
  pub send_result: Arc<Mutex<Option<SendResult>>>,
  pub is_pending: Arc<Mutex<bool>>,
  pub error: Option<String>,
  state: SendState,
  focus_context: FocusContext,
}

impl_has_focus_context!(WalletSend);

impl WalletSend {
  pub fn new() -> Self {
    Self {
      address: String::new(),
      amount: String::new(),
      wallet_secret: String::new(),
      payment_secret: String::new(),
      amount_sompi: None,
      send_result: Arc::new(Mutex::new(None)),
      is_pending: Arc::new(Mutex::new(false)),
      error: None,
      state: SendState::Details,
      focus_context: FocusContext { focus: FOCUS_NONE },
    }
  }

  pub fn reset(&mut self) {
    let send_result_clone = Arc::clone(&self.send_result);
    *send_result_clone.lock().unwrap() = None;

    self.address = String::new();
    self.amount = String::new();
    self.wallet_secret = String::new();
    self.payment_secret = String::new();
    self.amount_sompi = None;
    self.error = None;
    self.state = SendState::Details;
    self.assign_focus(Focus::Address);
  }

  fn validate_amount(&mut self) {
    if self.amount.is_empty() {
      self.amount_sompi = None;
      self.error = None;
      return;
    }

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
          self.error = Some(i18n("Please enter a valid amount of WALA").to_string());
        },
      },
      Err(err) => {
        self.amount_sompi = None;
        self.error = Some(err.to_string());
      }
    }
  }

  
  fn render_details_state(&mut self, ui: &mut egui::Ui) {
    ui.label(i18n("Recipient Address:"));
    let address_response = ui.add_sized(
      [ui.available_width(), 30.0],
      egui::TextEdit::singleline(
        &mut self.address,
      )
        .vertical_align(Align::Center)
        .frame(true)
    );
    self.next_focus(ui, Focus::Address, address_response.clone());
    ui.add_space(8.0);

    if address_response.lost_focus() && handle_enter_key(ui) {
      self.assign_focus(Focus::Amount);
    }

    let amount_before = self.amount.clone();
    ui.label(i18n("Amount:"));
    let amount_response = ui.add_sized(
      [ui.available_width(), 30.0],
      egui::TextEdit::singleline(
        &mut self.amount,
      )
        .vertical_align(Align::Center)
        .frame(true)
    );
    self.next_focus(ui, Focus::Amount, amount_response.clone());

    if amount_before != self.amount {
      self.validate_amount();
    }

    ui.add_space(16.0);

    let enabled = !self.address.trim().is_empty() 
      && self.address.chars().all(|c| c.is_alphanumeric() || c == ':')
      && self.amount_sompi.is_some()
      && self.error.is_none();

    if amount_response.lost_focus() && handle_enter_key(ui) {
      if enabled {
        self.state = SendState::Confirm;
        self.assign_focus(Focus::WalletSecret);
      } else {
        self.assign_focus(Focus::Amount);
      }
    }

    if ui.dx_large_button_enabled(
      enabled,
      i18n("Next"),
    ).clicked() {
      self.state = SendState::Confirm;
      self.assign_focus(Focus::WalletSecret);
    }
  }

  fn render_confirm_state(&mut self, ui: &mut egui::Ui, core: &mut Core) {
    let requires_bip39_passphrase = core.current_account.clone().unwrap()
      .requires_bip39_passphrase(&core.clone());

    ui.label(i18n("Wallet Password:"));
    let wallet_response = ui.add_sized(
      [ui.available_width(), 30.0],
      egui::TextEdit::singleline(
        &mut self.wallet_secret,
      )
        .vertical_align(Align::Center)
        .password(true)
        .frame(true)
    );
    self.next_focus(ui, Focus::WalletSecret, wallet_response.clone());
    ui.add_space(8.0);

    let mut enabled = true;
    if let Some(error) = &self.error {
      enabled = false;
    }

    enabled &= !self.address.trim().is_empty() && 
      !self.amount_sompi.is_none() && 
      !self.wallet_secret.trim().is_empty() &&
      (self.payment_secret.trim().is_empty() ^ requires_bip39_passphrase)
    ;

    if wallet_response.lost_focus() && handle_enter_key(ui) {
      if requires_bip39_passphrase {
        self.assign_focus(Focus::PaymentSecret);
      } else {
        if !*self.is_pending.lock().unwrap() {
          if enabled {
            let send_result_clone = Arc::clone(&self.send_result);
            *send_result_clone.lock().unwrap() = None;
            self.send_funds(core, self.amount_sompi.unwrap());
          } else {
            self.assign_focus(Focus::Amount);
          }
        }
      }
    }

    if requires_bip39_passphrase {
      ui.label(i18n("Payment Secret:"));
      let payment_response = ui.add_sized(
        [ui.available_width(), 30.0],
        egui::TextEdit::singleline(
          &mut self.payment_secret,
        )
          .vertical_align(Align::Center)
          .password(true)
          .frame(true)
      );
      self.next_focus(ui, Focus::WalletSecret, payment_response.clone());

      if payment_response.lost_focus() && handle_enter_key(ui) {
        if enabled {
          if !*self.is_pending.lock().unwrap() {
            let send_result_clone = Arc::clone(&self.send_result);
            *send_result_clone.lock().unwrap() = None;
            self.send_funds(core, self.amount_sompi.unwrap());
          }
        } else {
          self.assign_focus(Focus::WalletSecret);
        }
      }
    } else {
      ui.add_space(30.0);
    }

    ui.add_space(16.0);

    ui.horizontal(|ui| {
      let button_width = 130.0;
      ui.add_space((ui.available_width() - (button_width * 2.0 + 8.0)) / 2.0);

      if ui.dx_button_sized(
        i18n("Back"),
        24.0,
        Default::default(),
        vec2(button_width, 40.0),
      ).clicked() {
        self.state = SendState::Details;
      }
      ui.add_space(8.0);

      if ui.dx_button_sized_enabled(
        enabled,
        i18n("Send"),
        24.0,
        Default::default(),
        vec2(button_width, 40.0),
      ).clicked() {
        if !*self.is_pending.lock().unwrap() {
          let send_result_clone = Arc::clone(&self.send_result);
          *send_result_clone.lock().unwrap() = None;
          self.send_funds(core, self.amount_sompi.unwrap());
        }
      }
    });
  }

  fn render_success_state(&mut self, ui: &mut egui::Ui) {
    let window_rect = ui.min_rect();
    let coin_diameter = 200.0;
    let image_pos = pos2(
      window_rect.center().x,
      window_rect.min.y + (coin_diameter + 20.0) / 2.0
    );

    DXImage::draw(ui, &Assets::get().wala_coin, coin_diameter, image_pos, egui::Align2::CENTER_CENTER);
    ui.add_space(20.0);

    ui.heading(i18n("Transaction Submitted!"));
    ui.add_space(10.0);

    // Scroll area for transaction IDs
    let text_height = ui.text_style_height(&egui::TextStyle::Body);
    let scroll_area_height = 3.0 * text_height; // Adjust as needed
    egui::ScrollArea::vertical().max_height(scroll_area_height).show(ui, |ui| {
      if let Some(Ok(response)) = self.send_result.lock().unwrap().as_ref() {
        for (index, tx_id) in response.transaction_ids.iter().enumerate() {
          let tx_id_str = tx_id.to_string();
          let truncated_id = format!(
            "TX {}: {}...", 
            index + 1, 
            &tx_id_str.as_str()[..32.min(tx_id_str.len())]
          );
          
          let link = format!("https://explorer.waglayla.com/txs/{}", tx_id);

          let response = ui.add(
            egui::Label::new(
              egui::RichText::new(truncated_id)
                .underline()
                .color(theme_color().strong_color)
            )
            .sense(egui::Sense::click())
          )
          .on_hover_cursor(egui::CursorIcon::PointingHand)
          ;

          if response.clicked() {
            if let Err(err) = open::that(&link) {
              log_error!("Failed to open URL: {}", err);
            }
          }

          response.on_hover_text_at_pointer(&link);
        }
      }
    });
    ui.add_space(20.0);

    if ui.dx_large_button(i18n("Done")).clicked() {
      self.reset();
    }
  }

  pub fn render(
    &mut self,
    open: &mut bool,
    core: &mut Core,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
  ) {
    let mut F = create_custom_popup(ctx);
    F.rounding = 10.0.into();

    let h_scale = max(450, (ui.available_width() / 1.5) as i32);
    let v_scale = max(450, (ui.available_height() / 1.33) as i32);

    let screen_rect = ctx.screen_rect();
    let default_pos = egui::Pos2 {
      x: screen_rect.center().x - (h_scale as f32 / 2.0).min(300.0),
      y: screen_rect.center().y - (v_scale as f32 / 2.0),
    };

    egui::Window::new(i18n("Send WALA"))
      .open(open)
      .collapsible(true)
      .resizable(false)
      .default_pos(default_pos)
      .max_size([300.0, v_scale as f32])
      .frame(F)
      .show(ui.ctx(), |ui| {
        egui::Frame::none()
          .inner_margin(12.0)
          .show(ui, |ui| {
            ui.vertical_centered(|ui| {
              // Upper half (spinner/logo area)
              if *self.is_pending.lock().unwrap() {
                ui.add_space(16.0);
                ui.add(DX_Spinner::new()
                  .size(200.0)
                  .color(theme_color().strong_color)
                  .stroke_width(12.0)
                );
              } else {
                if let Some(result) = self.send_result.lock().unwrap().as_ref() {
                  match result {
                    Ok(response) => {
                      self.state = SendState::Success;
                    }
                    Err(err) => {
                      ui.label(i18n("Transaction failed."));
                      ui.colored_label(egui::Color32::RED, format!("{:?}", err));
                    }
                  }
                } else {
                  let window_rect = ui.min_rect();

                  let coin_diameter = 200.0;
                  let image_pos = pos2(
                    window_rect.center().x,
                    window_rect.min.y + (coin_diameter + 20.0) / 2.0
                  );
              
                  DXImage::draw(ui, &Assets::get().wala_coin, coin_diameter, image_pos, egui::Align2::CENTER_CENTER);

                  ui.add_space(20.0)
                }
              }

              match self.state {
                SendState::Details => self.render_details_state(ui),
                SendState::Confirm => self.render_confirm_state(ui, core),
                SendState::Success => self.render_success_state(ui),
              }

              if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::RED, error);
              } else {
                ui.label("");
              }
            });
          });
      });
  }

  fn send_funds(&mut self, core: &mut Core, amount: u64) {
    let address = self.address.clone();
    let wallet = manager().wallet();
    let send_result_clone = Arc::clone(&self.send_result);
    let is_pending_clone = Arc::clone(&self.is_pending);

    *self.is_pending.lock().unwrap() = true;

    let payment_output = PaymentOutput {
      address: Address::try_from(address).unwrap(),
      amount,
    };

    let core_delegate = core.clone();
    let self_clone = self.clone();

    tokio::spawn(async move {
      let request = AccountsSendRequest {
        account_id: core_delegate.current_account.unwrap().id(),
        destination: PaymentDestination::from(payment_output),
        priority_fee_sompi: Fees::from(0_u64),
        wallet_secret: Secret::from(self_clone.wallet_secret),
        payment_secret: Some(Secret::from(self_clone.payment_secret)),
        payload: None,
      };

      let result = wallet.accounts_send_call(request).await;
      *send_result_clone.lock().unwrap() = Some(result);
      *is_pending_clone.lock().unwrap() = false;
    });
  }
}