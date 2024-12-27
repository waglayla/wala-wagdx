use crate::imports::*;

use tokio::time::{timeout, Duration};
use waglayla_wallet_core::tx::{GeneratorSummary, PaymentOutput, PaymentDestination, Fees};
use waglayla_wallet_core::api::{AccountsSendRequest, AccountsSendResponse};
use std::sync::{Arc, Mutex};
use core::cmp::max;

define_indexed_enum!(
  Focus,
  WalletSecret,
  PaymentSecret
);

#[derive(Clone, Default)]
pub struct WalletSweep {
  pub wallet_secret: String,
  pub payment_secret: String,
  pub send_result: Arc<Mutex<Option<SendResult>>>,
  pub is_pending: Arc<Mutex<bool>>,
  focus_context: FocusContext,
}

impl_has_focus_context!(WalletSweep);

impl WalletSweep {
  pub fn new() -> Self {
    Self {
      wallet_secret: String::new(),
      payment_secret: String::new(),
      send_result: Arc::new(Mutex::new(None)),
      is_pending: Arc::new(Mutex::new(false)),
      focus_context: FocusContext { focus: FOCUS_NONE },
    }
  }

  pub fn reset(&mut self) {
    let send_result_clone = Arc::clone(&self.send_result);
    *send_result_clone.lock().unwrap() = None;

    self.wallet_secret.zeroize();
    self.payment_secret.zeroize();
    self.assign_focus(Focus::WalletSecret);
  }

  pub fn render(
    &mut self,
    open: &mut bool,
    core: &mut Core,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
  ) {
      let requires_bip39_passphrase = core.current_account.clone().unwrap()
        .requires_bip39_passphrase(&core.clone());
      let mut F = create_custom_popup(ctx);
      F.rounding = 10.0.into();

      let h_scale = max(450, (ui.available_width() / 1.5) as i32);
      let v_scale = max(450, (ui.available_height() / 1.33) as i32);

      let screen_rect = ctx.screen_rect();
      let default_pos = egui::Pos2 {
        x: screen_rect.center().x - (h_scale as f32 / 2.0),
        y: screen_rect.center().y - (v_scale as f32 / 2.0),
      };

      egui::Window::new(i18n("Compound UTXOs"))
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
                let mut finish = false;
                if *self.is_pending.lock().unwrap() {
                  ui.add_space(16.0);
                  ui.add(DX_Spinner::new()
                    .size(200.0)
                    .color(theme_color().strong_color)
                    .stroke_width(12.0)
                  );
                } else{
                  let response = ui.add(
                    egui::Label::new(
                      egui::RichText::new(egui_phosphor::bold::ARROWS_MERGE)
                        .font(egui::FontId::new(200.0, egui::FontFamily::Name("phosphor-fill".into())))
                        .color(theme_color().strong_color),
                    )
                  )
                  .on_hover_cursor(egui::CursorIcon::Help)
                  .on_hover_text_at_pointer(i18n("Compounding or sweeping your wallet \
                    allows you to combine several smaller UTXOs into larger ones. \
                    This may allow you to send more WALA than you otherwise could, \
                    due to transaction size limits."));

                  if let Some(result) = self.send_result.lock().unwrap().as_ref() {
                    match result {
                      Ok(response) => {
                        ui.heading(i18n("Success!"));
                        finish = true;
                      }
                      Err(err) => {
                        ui.label(i18n("Transaction(s) failed."));
                        ui.colored_label(egui::Color32::RED, format!("{:?}", err.to_string()));
                      }
                    }
                  }
                }

                if !finish {
                  let enabled =
                    !self.wallet_secret.is_empty() &&
                    (self.payment_secret.trim().is_empty() ^ requires_bip39_passphrase)
                  ;

                  ui.label(i18n("Wallet Password:"));
                  let wallet_secret_response = ui.add_sized(
                    [ui.available_width(), 30.0],
                    egui::TextEdit::singleline(&mut self.wallet_secret)
                      .vertical_align(Align::Center)
                      .password(true),
                  );
                  self.next_focus(ui, Focus::WalletSecret, wallet_secret_response.clone());
                  ui.add_space(12.0);

                  if wallet_secret_response.lost_focus() && handle_enter_key(ui) {
                    if requires_bip39_passphrase {
                      self.assign_focus(Focus::PaymentSecret);
                    } else {
                      if enabled && !*self.is_pending.lock().unwrap() {
                        let send_result_clone = Arc::clone(&self.send_result);
                        *send_result_clone.lock().unwrap() = None;
                        self.sweep(core);
                      }
                    }
                  }

                  if requires_bip39_passphrase {
                    ui.label(i18n("Payment Secret:"));
                    let payment_secret_response = ui.add_sized(
                      [ui.available_width(), 30.0],
                      egui::TextEdit::singleline(&mut self.payment_secret)
                        .vertical_align(Align::Center)
                        .password(true),
                    );
                    self.next_focus(ui, Focus::PaymentSecret, payment_secret_response.clone());

                    if payment_secret_response.lost_focus() && handle_enter_key(ui) {
                      if enabled && !*self.is_pending.lock().unwrap() {
                        let send_result_clone = Arc::clone(&self.send_result);
                        *send_result_clone.lock().unwrap() = None;
                        self.sweep(core);
                      }
                    }
                  } else {
                    ui.add_space(30.0);
                  }

                  ui.add_space(16.0);

                  if ui
                    .dx_large_button_enabled(
                      enabled && !finish,
                      i18n("Confirm"),
                    )
                    .clicked()
                  {
                    if enabled && !*self.is_pending.lock().unwrap() {
                      let send_result_clone = Arc::clone(&self.send_result);
                      *send_result_clone.lock().unwrap() = None;
                      self.sweep(core);
                    }
                  }
                } else {
                  ui.add_space(92.0);
                  if ui.dx_large_button(i18n("Done")).clicked()
                  {
                    self.reset();
                  }
                }
              });
            });
        });
  }

  fn sweep(&mut self, core: &mut Core) {
    use std::panic;

    let core_delegate = core.clone();
    let wallet = manager().wallet();
    let send_result_clone = Arc::clone(&self.send_result);
    let is_pending_clone = Arc::clone(&self.is_pending);

    let self_clone = self.clone();

    *self.is_pending.lock().unwrap() = true;

    tokio::spawn(async move {
      let result = panic::AssertUnwindSafe(async {
        let request = AccountsSendRequest {
          account_id: core_delegate.current_account.unwrap().id(),
          destination: PaymentDestination::Change,
          priority_fee_sompi: Fees::None,
          wallet_secret: Secret::from(self_clone.wallet_secret),
          payment_secret: Some(Secret::from(self_clone.payment_secret)),
          payload: None,
        };

        let timeout_duration = Duration::from_secs(12); // Set your desired timeout duration here

        let result = wallet.accounts_send_call(request).await;

        match result {
          Ok(call_result) => {
            *send_result_clone.lock().unwrap() = Some(Ok(call_result));
          }
          Err(err) => {
            *send_result_clone.lock().unwrap() = Some(Err(err));
          }
        }
      })
      .catch_unwind()
      .await;

      match result {
        Ok(_) => {}
        Err(_) => {
          *send_result_clone.lock().unwrap() = Some(
            Err(waglayla_wallet_core::error::Error::Custom(
              i18n("Unknown Error when Compounding... Please check your UTXO count, or try again").to_string()
            ))
          );
        }
      }

      *is_pending_clone.lock().unwrap() = false;
    });
  }
}
