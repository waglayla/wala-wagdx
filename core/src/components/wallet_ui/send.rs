use crate::imports::*;
use waglayla_wallet_core::tx::{GeneratorSummary, PaymentOutput, PaymentDestination, Fees};
use waglayla_wallet_core::api::{AccountsSendRequest, AccountsSendResponse};
use std::sync::{Arc, Mutex};
use core::cmp::max;

pub type SendResult = std::result::Result<AccountsSendResponse, waglayla_wallet_core::error::Error>;

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
}

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
    }
  }

  pub fn reset(&mut self) {
    let send_result_clone = Arc::clone(&self.send_result);
    *send_result_clone.lock().unwrap() = None;

    self.wallet_secret = "".to_string();
    self.payment_secret = "".to_string();
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
        x: screen_rect.center().x - (h_scale as f32 / 2.0),
        y: screen_rect.center().y - (v_scale as f32 / 2.0),
      };

      egui::Window::new(i18n("Send Waglayla"))
        .open(open)
        .collapsible(true)
        .resizable(false)
        .default_pos(default_pos)
        .max_size([h_scale as f32, v_scale as f32])
        .frame(F)
        .show(ui.ctx(), |ui| {
          egui::Frame::none()
            .inner_margin(12.0)
            .show(ui, |ui| {
              ui.vertical_centered(|ui| {
                // Pending spinner
                if *self.is_pending.lock().unwrap() {
                  ui.add_space(16.0);
                  ui.add(DX_Spinner::new()
                    .size(200.0)
                    .color(theme_color().strong_color)
                    .stroke_width(12.0)
                  );
                } else{
                  if let Some(result) = self.send_result.lock().unwrap().as_ref() {
                    match result {
                      Ok(response) => {
                        ui.heading(i18n("Transaction successful!"));
                        ui.label("");
                        ui.label(format!("Transaction IDs: {:?}", response.transaction_ids));
                      }
                      Err(err) => {
                        ui.label(i18n("Transaction failed."));
                        ui.colored_label(egui::Color32::RED, format!("{:?}", err));
                      }
                    }
                  } else {
                    ui.add_space(220.0);
                  }
                }

                egui::ScrollArea::vertical()
                .max_height(125.0)
                .show(ui, |ui| {
                  // Address field with icon
                  ui.label(i18n("Recipient Address:"));
                  ui.horizontal(|ui| {
                    // ui.add(egui::Image::new(ph_icon("Address")));  // Or similar appropriate icon
                    ui.add_sized(
                      [ui.available_width(), 30.0],  // Fixed height for consistency
                      egui::TextEdit::singleline(&mut self.address)
                        .vertical_align(Align::Center)
                        .hint_text("Enter recipient address")  // Helper text
                        .frame(true)  // Add visible frame
                    );
                  });
                  ui.add_space(8.0);
              
                  // Amount field with icon
                  ui.label(i18n("Amount:"));
                  ui.horizontal(|ui| {
                    // ui.add(egui::Image::new(ph_icon("Coin")));  // Or similar appropriate icon
                    ui.add_sized(
                      [ui.available_width(), 30.0],
                      egui::TextEdit::singleline(&mut self.amount)
                        .vertical_align(Align::Center)
                        .hint_text("Enter amount")
                        .frame(true)
                    );
                  });
                  ui.add_space(8.0);
              
                  // Wallet Secret field with icon
                  ui.label(i18n("Wallet Secret:"));
                  ui.horizontal(|ui| {
                    // ui.add(egui::Image::new(ph_icon("Key")));  // Or similar appropriate icon
                    ui.add_sized(
                      [ui.available_width(), 30.0],
                      egui::TextEdit::singleline(&mut self.wallet_secret)
                        .vertical_align(Align::Center)
                        .password(true)
                        .hint_text("Enter wallet secret")
                        .frame(true)
                    );
                  });
                  ui.add_space(8.0);
              
                  // Payment Secret field with icon
                  ui.label(i18n("Payment Secret:"));
                  ui.horizontal(|ui| {
                    // ui.add(egui::Image::new(ph_icon("LockKey")));  // Or similar appropriate icon
                    ui.add_sized(
                      [ui.available_width(), 30.0],
                      egui::TextEdit::singleline(&mut self.payment_secret)
                        .vertical_align(Align::Center)
                        .password(true)
                        .hint_text("Enter payment secret")
                        .frame(true)
                    );
                  });
                  ui.add_space(16.0);
              });

                let mut enabled = true;

                // Error message
                if let Some(error) = &self.error {
                  enabled = false;
                  ui.colored_label(egui::Color32::RED, error);
                  ui.add_space(8.0);
                }

                // Send button
                if ui
                  .dx_large_button_enabled(
                    !self.address.trim().is_empty() && !self.amount_sompi.is_none() && enabled,
                    i18n("Send"),
                  )
                  .clicked()
                {
                  if !*self.is_pending.lock().unwrap() {
                    let send_result_clone = Arc::clone(&self.send_result);
                    *send_result_clone.lock().unwrap() = None;

                    self.send_funds(core, self.amount_sompi.unwrap());
                  }
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
