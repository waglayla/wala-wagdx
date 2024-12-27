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
  pub transactions: WalletTransactions,
}

impl ViewWallet {
  pub fn new(manager: DX_Manager) -> Self {
    Self {
      manager,
      state: State::Info,
      message: None,
      info: Default::default(),
      transactions: Default::default(),
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
      ui.set_style(style);
      ui.vertical_centered(|ui| {
        self.info.render(core, ctx, ui);
        ui.add_space(16.0);
        self.render_tx_header(ui);
        ui.add_space(56.0);
        self.transactions.render(core, ctx, ui);
      });
    });
  }
}

impl ViewWallet {
  fn render_tx_header(&self, ui: &mut egui::Ui) {
    let text = i18n("Transaction History");
    let font_id = egui::FontId::new(50.0, get_font_family("DINishCondensed", true, false));
    let galley = ui.fonts(|f| {
      f.layout_no_wrap(text.to_string(), font_id.clone(), theme_color().strong_color)
    });

    let available_rect = ui.available_rect_before_wrap();
    let text_pos = egui::pos2(
      available_rect.center().x - galley.size().x / 2.0,
      available_rect.top() - 12.0,
    );

    let painter = ui.painter();
    painter.text(
      text_pos,
      egui::Align2::LEFT_TOP,
      text,
      egui::FontId::new(50.0, get_font_family("DINishCondensed", true, false)),
      theme_color().strong_color,
    );
  }
}