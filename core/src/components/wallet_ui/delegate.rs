use crate::imports::*;

#[derive(Default, Clone)]
pub struct WalletDelegator {}

impl ComponentT for WalletDelegator {
  fn name(&self) -> Option<&'static str> {
    Some("Delegator")
  }

  fn render(
    &mut self,
    core: &mut Core,
    _ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    _ui: &mut egui::Ui,
  ) {
    let has_account = core.current_account.is_some();
    if has_account {
      core.set_active_component::<wallet_ui::ViewWallet>();
    } else {
      core.set_active_component::<wallet_ui::CreateWallet>();
    }
  }
}