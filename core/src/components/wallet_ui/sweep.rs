use crate::imports::*;
use waglayla_wallet_core::tx::{GeneratorSummary, PaymentOutput, PaymentDestination, Fees};
use waglayla_wallet_core::api::{AccountsSendRequest, AccountsSendResponse};
use std::sync::{Arc, Mutex};
use core::cmp::max;

pub type SendResult = std::result::Result<AccountsSendResponse, waglayla_wallet_core::error::Error>;

#[derive(Clone, Default)]
pub struct WalletSweep {
  pub wallet_secret: String,
  pub payment_secret: String,
  pub send_result: Arc<Mutex<Option<SendResult>>>,
  pub is_pending: Arc<Mutex<bool>>,
  pub error: Option<String>,
}

impl WalletSweep {
  pub fn new() -> Self {
    Self {
      wallet_secret: String::new(),
      payment_secret: String::new(),
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
                        ui.colored_label(egui::Color32::RED, format!("{:?}", err));
                      }
                    }
                  }
                }

                if !finish {
                  // Input for Address
                  ui.label(i18n("Wallet Secret:"));
                  ui.add_sized(
                    [ui.available_width(), 30.0],
                    egui::TextEdit::singleline(&mut self.wallet_secret)
                      .vertical_align(Align::Center)
                      .password(true),
                  );
                  ui.add_space(12.0);

                  // Input for Amount
                  ui.label(i18n("Payment Secret:"));
                  ui.add_sized(
                    [ui.available_width(), 30.0],
                    egui::TextEdit::singleline(&mut self.payment_secret)
                      .vertical_align(Align::Center)
                      .password(true),
                  );
                  ui.add_space(16.0);

                  if ui
                    .dx_large_button_enabled(
                      !self.wallet_secret.is_empty() && !finish,
                      i18n("Confirm"),
                    )
                    .clicked()
                  {
                    if !*self.is_pending.lock().unwrap() {
                      let send_result_clone = Arc::clone(&self.send_result);
                      *send_result_clone.lock().unwrap() = None;

                      self.sweep(core);
                    }
                  }
                } else {
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
    let core_delegate = core.clone();
    let wallet = manager().wallet();
    let send_result_clone = Arc::clone(&self.send_result);
    let is_pending_clone = Arc::clone(&self.is_pending);

    let self_clone = self.clone();

    *self.is_pending.lock().unwrap() = true;

    tokio::spawn(async move {
      let request = AccountsSendRequest {
        account_id: core_delegate.current_account.unwrap().id(),
        destination: PaymentDestination::Change,
        priority_fee_sompi: Fees::None,
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
