use crate::imports::*;
use core::cmp::max;


#[derive(Default, Clone, PartialEq)]
enum ExportState {
  #[default]
  SelectKey,
  Auth,
  Export,
  Exporting,
  Mnemonic { mnemonic : String },
  Error { error: String },
}

#[derive(Clone)]
pub enum ExportResult {
  Mnemonic(String),
}

define_indexed_enum!(
  Focus,
  WalletSecret,
  PaymentSecret
);

#[derive(Clone, Default)]
pub struct WalletExport {
  state : ExportState,
  wallet_secret : String,
  payment_secret: String,
  prv_key_data_info : Option<Arc<PrvKeyDataInfo>>,
  // kind: Enum TODO
  pub is_pending: Arc<Mutex<bool>>,
  pub export_result: Arc<Mutex<Option<Result<ExportResult>>>>,
  error : Option<String>,
  focus_context: FocusContext,
  mnemonic_presenter_context : MnemonicPresenterContext,
}

impl_has_focus_context!(WalletExport);

impl WalletExport {
  pub fn new() -> Self {
    Self { 
      state : ExportState::SelectKey,
      wallet_secret : String::new(),
      payment_secret : String::new(),
      prv_key_data_info : None,
      is_pending: Arc::new(Mutex::new(false)),
      export_result : Arc::new(Mutex::new(None)),
      error : None,
      focus_context: FocusContext { focus: Focus::WalletSecret.to_u32().unwrap() },
      mnemonic_presenter_context: Default::default(),
    }
  }

  pub fn reset(&mut self) {
    self.wallet_secret.zeroize();
    self.mnemonic_presenter_context.zeroize();
    self.assign_focus(Focus::WalletSecret);
    self.state = ExportState::SelectKey;
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

    let h_scale = max(500, (ui.available_width() / 1.5) as i32);
    let v_scale = max(450, (ui.available_height() / 1.33) as i32);

    let screen_rect = ctx.screen_rect();
    let default_pos = egui::Pos2 {
        x: screen_rect.center().x - (h_scale as f32 / 2.0),
        y: screen_rect.center().y - (v_scale as f32 / 2.0),
    };

    egui::Window::new(i18n("Export Wallet"))
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

            match &self.state {
              ExportState::SelectKey => {
                let prv_key_data_map = core.prv_key_data_map.clone();

                ui.heading(i18n("Please select the private key to export"));
                ui.add_space(12.0);

                if let Some(prv_key_data_map) = prv_key_data_map {
                  let item_count = prv_key_data_map.len();
                  let item_height = 48.0; // Height of each button
                  let total_item_height = item_count as f32 * item_height;
                  let max_scroll_height = ui.available_height() - 56.0;

                  // Dynamically calculate padding to center the content
                  let remaining_space = (v_scale as f32 - total_item_height - 12.0).max(0.0);
                  let vertical_padding = remaining_space / 2.0;

                  ui.add_space(vertical_padding.min(max_scroll_height / 2.0));
                  egui::ScrollArea::vertical()
                    .max_height(max_scroll_height)
                    .show(ui, |ui| {
                      for prv_key_data_info in prv_key_data_map.values() {
                        if ui.dx_button_sized(
                          prv_key_data_info.name_or_id(),
                          32.0,
                          Default::default(), 
                          vec2(220.0, 48.0)
                        ).clicked() {
                          self.prv_key_data_info = Some(prv_key_data_info.clone());
                          self.assign_focus(Focus::WalletSecret);
                          self.state = ExportState::Auth;
                        }
                        ui.label("");
                      }
                    });
                  ui.add_space(vertical_padding.min(max_scroll_height / 2.0));
                } else {
                  ui.add_space(400.0);
                }
              }

              ExportState::Auth => {
                let requires_bip39_passphrase = self.prv_key_data_info.as_ref().unwrap().requires_bip39_passphrase();
                let enabled = !self.wallet_secret.trim().is_empty() &&
                  (self.payment_secret.trim().is_empty() ^ requires_bip39_passphrase)
                ;
                ui.heading(i18n("Unlock Wallet"));
                ui.add_space(6.0);
                // Total content height (e.g., heading + spacing + fields)
                let mut content_height = 6.0 + 30.0; // Heading + Wallet Secret text field
                if requires_bip39_passphrase {
                    content_height += 12.0 + 30.0; // Add space + Payment Secret text field
                }
                let available_height = ui.available_height() - 56.0;
                let vertical_padding = ((available_height - content_height).max(0.0)) / 2.0;

                ui.add_space(vertical_padding);

                ui.label(i18n("Wallet Secret:"));
                let wallet_response = ui.add_sized(
                  [300.0, 30.0],
                  egui::TextEdit::singleline(
                    &mut self.wallet_secret,
                  )
                    .vertical_align(Align::Center)
                    .password(true)
                    .frame(true)
                );
                self.next_focus(ui, Focus::WalletSecret, wallet_response.clone());

                if wallet_response.lost_focus() && handle_enter_key(ui) {
                  if requires_bip39_passphrase {
                    self.assign_focus(Focus::PaymentSecret);
                  } else {
                    if enabled {
                      self.export(core);
                      self.state = ExportState::Export;
                    } else {
                      self.assign_focus(Focus::WalletSecret);
                    }
                  }
                }

                if requires_bip39_passphrase {
                  ui.label(i18n("Payment Secret:"));
                  let payment_secret_response = ui.add_sized(
                    [300.0, 30.0],
                    egui::TextEdit::singleline(&mut self.payment_secret)
                      .vertical_align(Align::Center)
                      .password(true),
                  );
                  self.next_focus(ui, Focus::PaymentSecret, payment_secret_response.clone());

                  if payment_secret_response.lost_focus() && handle_enter_key(ui) {
                    if enabled {
                      self.export(core);
                      self.state = ExportState::Export;
                    } else {
                      self.assign_focus(Focus::PaymentSecret);
                    }
                  }
                } else {
                  ui.add_space(30.0);
                }

                ui.add_space(vertical_padding);
              }

              ExportState::Export => {
                if *self.is_pending.lock().unwrap() {
                  ui.heading(i18n("Processing..."));
                  ui.add_space(16.0);
                  ui.add(DX_Spinner::new()
                    .size(ui.available_height() - 20.0)
                    .color(theme_color().strong_color)
                    .stroke_width((ui.available_height() - 20.0) / 12.0)
                  );
                  ui.add_space(16.0);
                } else {
                  if let Some(result) = self.export_result.lock().unwrap().as_ref() {
                    match result {
                      Ok(kind) => {
                        match kind {
                          ExportResult::Mnemonic(mnemonic) => {
                            self.state = ExportState::Mnemonic { mnemonic: mnemonic.to_string() };
                          }
                        }
                      }
                      Err(err) => {
                        self.state = ExportState::Error { error: err.to_string() };
                      }
                    }
                  }
                }
              }

              ExportState::Error { error } => {
                ui.heading(i18n("Error Exporting Wallet"));
                ui.add_space(8.0);

                ui.colored_label(egui::Color32::RED, format!("{:?}", error));

                if ui.dx_large_button(i18n("Restart")).clicked() {
                  self.state = ExportState::SelectKey;
                  self.assign_focus(Focus::WalletSecret);
                }
              }

              ExportState::Mnemonic { mnemonic } => {
                let available_height = ui.available_height();

                egui::ScrollArea::vertical()
                  .max_height(available_height - 60.) 
                  .show(ui, |ui| {
                    // render_centered_content_noback(ctx, ui, i18n("Mnemonic Seed Phrase"), |ui| {
                    let mut mnemonic_presenter = MnemonicPresenter::new(mnemonic.as_str(), &mut self.mnemonic_presenter_context);
    
                    ui.vertical_centered(|ui| {
                      ui.label(RichText::new(i18n(mnemonic_presenter.notice())).size(14.));
                      ui.label("");
                      ui.label(RichText::new(i18n(mnemonic_presenter.warning())).size(14.));
                    });
    
                    ui.label("");
                    mnemonic_presenter.render(core, ui, Some(i18n("Your base wallets mnemonic seed is:")));
                    ui.label("");
                });
    
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                  if ui.dx_large_button(i18n("Finish")).clicked() {
                    self.reset();
                  }
                  ui.add_space(4.);
                  ui.separator();
                  ui.add_space(4.);
                }); 
              }

              _ => {}
            }
          });
        });
      });
  }

  fn export(&mut self, core: &mut Core) {
    let wallet_secret = Secret::new(self.wallet_secret.as_str().into());
    let requires_bip39_passphrase = self.prv_key_data_info.as_ref().unwrap().requires_bip39_passphrase();
    let payment_secret: Option<Secret> = requires_bip39_passphrase
        .then(|| self.payment_secret.as_str().into());
    self.wallet_secret.zeroize();

    let wallet = manager().wallet().clone();
    let prv_key_data_info = self.prv_key_data_info.clone();
    let export_result_clone = Arc::clone(&self.export_result);
    let is_pending_clone = Arc::clone(&self.is_pending);

    *self.is_pending.lock().unwrap() = true;

    tokio::spawn(async move {
      let result: Result<ExportResult> = async {
        if let Some(prv_key_data_info) = prv_key_data_info {
          let prv_key_data = wallet.prv_key_data_get(*prv_key_data_info.id(), wallet_secret).await?;
          let mnemonic = prv_key_data.as_mnemonic(payment_secret.as_ref())?.ok_or(Error::custom("No mnemonic available"))?;
          Ok(ExportResult::Mnemonic(mnemonic.phrase_string()))
        } else {
          Err(Error::custom("No private key data available"))
        }
      }
      .await;

      *export_result_clone.lock().unwrap() = Some(result);
      *is_pending_clone.lock().unwrap() = false;
    });
  }
}