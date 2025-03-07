use crate::imports::*;
use crate::core::TRANSACTION_PAGE_SIZE;

#[derive(Default)]
pub struct WalletTransactions {
  pub start: u64,
}

impl WalletTransactions {
  pub fn render(&mut self, core : &mut Core, ctx: &egui::Context, ui: &mut Ui) {
    let account = core.current_account.as_ref().expect("No Account Selected").clone();
    let current_daa_score = core.node_state().current_daa_score();

    let max_height = ui.available_height() - (ui.fonts(|fonts|RichText::new("YWgy").font_height(fonts, ui.style())).at_least(ui.spacing().interact_size.y) * 2.0 + 5.0);

    egui::ScrollArea::vertical()
      .max_height(max_height)
      .auto_shrink([false, true])
      .id_source("tx_browser")
      .show(ui, |ui| {
        egui::Frame::none()
        .inner_margin(egui::Margin::same(12.0))
        .rounding(egui::Rounding::same(10.0))
        .fill(theme_color().button_color)
        .show(ui, |ui| {
          let transactions = account.transactions();
          if transactions.is_empty() {
            ui.vertical_centered(|ui| {
              ui.label("");
              ui.label(RichText::new(i18n("No transactions")).size(16.));
            });
          } else {
            let total: u64 = transactions.iter().map(|transaction| transaction.aggregate_input_value()).sum();
            transactions.iter().for_each(|transaction| {
              transaction.render(ui, current_daa_score, true, Some(total));
            });
          }
        });
    });

    let total_transactions = account.transaction_count();

    ui.add_space(6.);
    let pagination = Pagination::new(
      total_transactions,
      Some(self.start),
      Some(TRANSACTION_PAGE_SIZE),
      Some(13u64)                 
    );
    ui.add_space(12.);
    
    if let Some(start) = pagination.render(ui) {
      let end = (start + TRANSACTION_PAGE_SIZE as u64).min(total_transactions as u64);

      core.load_account_transactions_with_range(&account, start..end)
        .map_err(|err| {
          log_info!("Failed to load transactions\n{err:?}")
        })
        .ok();

      self.start = start;
      manager().request_repaint();
    }
  }

  pub fn reset(&mut self) {
    self.start = 0;
  }

  pub fn current_page(&self) -> u64 {
    self.start + 1
  }
}