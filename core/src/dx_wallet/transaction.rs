use crate::imports::*;
use egui_phosphor::light::*;
use waglayla_consensus_core::tx::{TransactionInput, TransactionOutpoint, TransactionOutput};
use waglayla_wallet_core::storage::{
    transaction::{TransactionData, UtxoRecord},
    TransactionKind,
};

use chrono::{DateTime, Local, TimeZone};

// For reference only
// /// @category Wallet SDK
// #[wasm_bindgen(inspectable)]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TransactionRecord {
//     pub id: TransactionId,
//     /// Unix time in milliseconds
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[serde(rename = "unixtimeMsec")]
//     #[wasm_bindgen(js_name = unixtimeMsec)]
//     pub unixtime_msec: Option<u64>,
//     pub value: u64,
//     #[wasm_bindgen(skip)]
//     pub binding: Binding,
//     #[serde(rename = "blockDaaScore")]
//     #[wasm_bindgen(js_name = blockDaaScore)]
//     pub block_daa_score: u64,
//     #[serde(rename = "network")]
//     #[wasm_bindgen(js_name = network)]
//     pub network_id: NetworkId,
//     #[serde(rename = "data")]
//     #[wasm_bindgen(skip)]
//     pub transaction_data: TransactionData,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[wasm_bindgen(getter_with_clone)]
//     pub note: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[wasm_bindgen(getter_with_clone)]
//     pub metadata: Option<String>,
// }

pub trait TX_Color {
  fn tx_color(&self) -> Color32;
}

#[derive(Debug)]
struct Context {
  record: Arc<TransactionRecord>,
  maturity: Option<bool>,
}

impl Context {
  pub fn new(record: Arc<TransactionRecord>, maturity: Option<bool>) -> Self {
    Self { record, maturity }
  }
}

struct Inner {
  id: TransactionId,
  context: Mutex<Arc<Context>>,
}

impl Inner {
  fn new(record: Arc<TransactionRecord>, maturity: Option<bool>) -> Self {
    Self {
      id: *record.id(),
      context: Mutex::new(Arc::new(Context::new(record, maturity))),
    }
  }
}

#[derive(Clone)]
pub struct Transaction {
  inner: Arc<Inner>,
}

const IN_COLOR: Color32 = Color32::from_rgb(0,130,39);
const OUT_COLOR: Color32 = Color32::from_rgb(162,0,0);

impl Transaction {
  pub fn new_confirmed(record: Arc<TransactionRecord>) -> Self {
    Self {
      inner: Arc::new(Inner::new(record, Some(true))),
    }
  }

  pub fn new_processing(record: Arc<TransactionRecord>) -> Self {
    Self {
      inner: Arc::new(Inner::new(record, Some(false))),
    }
  }

  pub fn new(record: Arc<TransactionRecord>) -> Self {
    Self {
      inner: Arc::new(Inner::new(record, None)),
    }
  }

  fn context(&self) -> Arc<Context> {
    self.inner.context.lock().unwrap().clone()
  }

  pub fn id(&self) -> TransactionId {
    self.inner.id
  }

  // pub fn date(&self) -> String {
  //   let unix_timestamp = self.inner.context.record.unixtime_msec;
  //   let timestamp_seconds = unix_timestamp / 1000;
  //   let datetime: DateTime<Local> = Local.timestamp_opt(timestamp_seconds, 0).unwrap();
  //   datetime.format("%Y-%m-%d %H:%M:%S").to_string()
  // }

  pub fn aggregate_input_value(&self) -> u64 {
    self.context().record.aggregate_input_value()
  }
}

impl IdT for Transaction {
  type Id = TransactionId;

  fn id(&self) -> &Self::Id {
    &self.inner.id
  }
}

impl std::fmt::Debug for Transaction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    std::fmt::Debug::fmt(&self.context(), f)
  }
}

pub type TransactionCollection = Collection<TransactionId, Transaction>;

impl From<TransactionRecord> for Transaction {
  fn from(record: TransactionRecord) -> Self {
    Self {
      inner: Arc::new(Inner::new(Arc::new(record), None)),
    }
  }
}

impl From<Arc<TransactionRecord>> for Transaction {
  fn from(record: Arc<TransactionRecord>) -> Self {
    Self {
      inner: Arc::new(Inner::new(record, None)),
    }
  }
}

impl Transaction {
  pub fn maturity(&self) -> bool {
    let Context { record, maturity } = &*self.context();
    maturity.expect("No Maturity Set")
  }

  pub fn render(
    &self,
    ui: &mut Ui,
    current_daa_score: Option<u64>,
    _include_utxos: bool,
    largest: Option<u64>,
  ) {
    let Context { record, maturity } = &*self.context();
    let tx_data = &record.transaction_data;
    let (sign, kind) = match tx_data.kind() {
      TransactionKind::Reorg => ("", "REORG"),
      TransactionKind::Stasis => ("", "STASIS"),
      TransactionKind::Incoming => ("", "INCOMING"),
      TransactionKind::External => ("", "EXTERNAL"),
      TransactionKind::Outgoing => ("-", "OUTGOING"),
      TransactionKind::Batch => ("", "BATCH"),
      TransactionKind::TransferIncoming => ("", "TRANSFER"),
      TransactionKind::TransferOutgoing => ("-", "TRANSFER"),
      TransactionKind::Change => ("", "CHANGE"),
    };

    let truncated_id = format!(
      "{}...", 
      &self.id().to_string().as_str()[..32]
    );

    let factor = (ui.available_width() / (764.0)).min(1.5);
    let cell_widths = [118.0 * factor, 285.0 * factor, 110.0 * factor, 87.0 * factor, 85.0 * factor];
    let font_size = 14.0 * factor;

    let total_width: f32 = cell_widths.iter().sum();

    ui.vertical_centered(|ui| {
      ui.set_width(total_width + 28.0);

      let painter = ui.painter();

      let rect = ui.available_rect_before_wrap();
      let rounded_rect = egui::Rect::from_min_size(
        rect.min - vec2(17.0, 0.0),
        egui::vec2(total_width + 56.0, font_size + 3.0),
      );

      let response = ui.interact(
        rounded_rect,
        egui::Id::new(format!("rounded_rect_ghost_tx_{}",self.id().to_string())), 
        egui::Sense::click(),
      ).on_hover_cursor(egui::CursorIcon::PointingHand);

      if response.clicked() {
        let link = format!("https://explorer.waglayla.com/txs/{}", self.id().to_string());
        if let Err(err) = open::that(&link) {
          log_error!("Failed to open URL: {}", err);
        }
      }

      let color_buffer = if response.hovered() {
        theme_color().fg_color
      } else {
        theme_color().button_color
      };

      let rect_color = ui.ctx().animate_color_with_time(
        egui::Id::new(format!("hover_color_tx_{}",self.id().to_string())),
        color_buffer,
        0.125
      );  

      painter.rect_filled(
        rounded_rect,
        5.0,
        rect_color,
      );

      ui.horizontal(|ui| {
        for &width in &cell_widths {
          ui.allocate_ui(egui::vec2(width, ui.available_height()), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
              match width {
                // Assign content for each specific cell
                w if w == cell_widths[0] => {
                  ui.add(Label::new(
                    RichText::new(record.unixtime_as_locale_string().expect("No Date Found for TX"))
                      .size(font_size),
                  ).selectable(false));
                }
                w if w == cell_widths[1] => {
                  ui.add(Label::new(
                    RichText::new(truncated_id.clone()).size(font_size),
                  ).selectable(false));
                }
                w if w == cell_widths[2] => {
                  match record.transaction_data() {
                    TransactionData::Reorg { .. }
                    | TransactionData::Stasis { .. }
                    | TransactionData::Incoming { .. }
                    | TransactionData::TransferIncoming { .. }
                    | TransactionData::External { .. } => {
                      ui.add(Label::new(
                        RichText::new(format!("{}", format_balance_tx(record.value())))
                          .color(IN_COLOR)
                          .size(font_size),
                      ).selectable(false));
                    }
                    TransactionData::Outgoing { payment_value, .. }
                    | TransactionData::TransferOutgoing { payment_value, .. } => {
                      if let Some(payment_value) = payment_value {
                        ui.add(Label::new(
                          RichText::new(format!("-{}", format_balance_tx(record.value())))
                            .color(OUT_COLOR)
                            .size(font_size),
                        ).selectable(false));
                      } else {
                        ui.add(Label::new(
                          RichText::new(format!("{}", format_balance_tx(record.value())))
                            .size(font_size),
                        ).selectable(false));
                      }
                    }
                    TransactionData::Batch { .. } => {
                      let aggregate_input_value = record.aggregate_input_value();
                      ui.add(Label::new(
                        RichText::new(format!("-{}", format_balance_tx(aggregate_input_value)))
                          .size(font_size),
                      ).selectable(false));
                    }
                    _ => {}
                  }
                }
                w if w == cell_widths[3] => {
                  match record.transaction_data() {
                    TransactionData::Reorg { .. }
                    | TransactionData::Stasis { .. }
                    | TransactionData::Incoming { .. }
                    | TransactionData::TransferIncoming { .. }
                    | TransactionData::External { .. } => {
                      if record.is_coinbase() {
                        ui.add(Label::new(
                          RichText::new("COINBASE").size(font_size),
                        ).selectable(false));
                      } else {
                        ui.add(Label::new(
                          RichText::new(kind).size(font_size),
                        ).selectable(false));
                      }
                    }
                    TransactionData::Outgoing { payment_value, .. }
                    | TransactionData::TransferOutgoing { payment_value, .. } => {
                      if let Some(payment_value) = payment_value {
                        ui.add(Label::new(
                          RichText::new(kind).size(font_size),
                        ).selectable(false));
                      } else {
                        ui.add(Label::new(
                          RichText::new("COMPOUND").size(font_size),
                        ).selectable(false));
                      }
                    }
                    TransactionData::Batch { .. } => {
                      ui.add(Label::new(
                        RichText::new("SWEEP").size(font_size),
                      ).selectable(false));
                    }
                    _ => {}
                  }
                }
                w if w == cell_widths[4] => {
                  if !maturity.unwrap_or(true) {
                    ui.add(Label::new(
                      RichText::new("PENDING").size(font_size),
                    ).selectable(false));
                  } else {
                    ui.add(Label::new(
                      RichText::new("COMPLETE").size(font_size),
                    ).selectable(false));
                  }
                }
                _ => {}
              }
            });
          });
        }

        // ui.painter().rect_stroke(
        //   rect.response.rect,
        //   0.0,                             // Corner rounding
        //   egui::Stroke::new(1.0, egui::Color32::BLUE), // Border width and color
        // );
      });
    });
  }
}