use crate::imports::*;
use super::*;

use waglayla_wallet_core::{wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, encryption::EncryptionKind, api::{AccountsDiscoveryRequest, AccountsDiscoveryKind}};
use waglayla_bip32::{WordCount, Mnemonic, Language};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub enum State {
  #[default]
  Info,
  Send,
  Receive,
  Compound
}

pub struct ViewWallet {
  #[allow(dead_code)]
  manager: DX_Manager,
  pub state: State,
  pub message: Option<String>,
  pub info: WalletBiscuit,
  // pub tx_result: Arc<Mutex<Option<UnlockResult>>>,
  pub is_pending: Arc<Mutex<bool>>,
}

impl ViewWallet {
  pub fn new(manager: DX_Manager) -> Self {
    Self {
      manager,
      state: State::Info,
      message: None,
      info: Default::default(),
      // tx_result: Arc::new(Mutex::new(None)),
      is_pending: Arc::new(Mutex::new(false)),
    }
  }

  pub fn update_biscuit_account(
    &mut self, 
    ctx: &egui::Context, 
    current_account: &Option<Account>,
    wallet_descriptor: &Option<WalletDescriptor>,
  ) {
    self.info.update_account_font_size(
      ctx,
      current_account,
      wallet_descriptor,
    );
  }
}

impl ComponentT for ViewWallet {
  fn name(&self) -> Option<&'static str> {
    Some("Wallet Orverview")
  }

  fn render(
    &mut self,
    core: &mut Core,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    *ui.spacing_mut() = Default::default();
    let mut style = (*ctx.style()).clone();
    style.spacing.window_margin = egui::Margin::same(20.0);

    egui::Frame::none()
      .inner_margin(20.0)
      .show(ui, |ui| 
    {
      egui::ScrollArea::vertical()
        .show(ui, |ui| 
      {
        ui.set_style(style);
        ui.vertical_centered(|ui| {
          self.info.render(core, ctx, ui)
        });
      });
    });
  }
}