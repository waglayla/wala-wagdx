use super::*;

pub fn describe_sync(state: NodeState) -> ( String, Color32) {
  if state.is_connected() {
    match state.sync_state.unwrap() {
      SyncState::Headers { progress, .. } => {
        (format!("{} {}%", i18n("Syncing Headers..."), progress), theme_color().separator_color)
      }
      SyncState::Blocks { progress, .. } => {
        (format!("{} {}%", i18n("Processing Blocks..."), progress), theme_color().separator_color)
      }
      SyncState::UtxoSync { chunks, total } => {
        let percentage = if total > 0 {
          (chunks as f64 / total as f64 * 100.0).round() as u32
        } else {
          0
        };
        (format!("{} {}%", i18n("Processing UTXOs..."), percentage), theme_color().separator_color)
      }
      SyncState::UtxoResync { .. } => {
        (i18n("Syncing UTXOs").to_string(), theme_color().strong_color)
      }
      SyncState::TrustSync { processed, total } => {
        let percentage = if total > 0 {
          (processed as f64 / total as f64 * 100.0).round() as u32
        } else {
          0
        };
        (format!("{} {}%", i18n("Validating..."), percentage), theme_color().separator_color)
      }
      SyncState::Synced => {
        (i18n("Fully Synced").to_string(), theme_color().strong_color)
      }
      SyncState::Proof { .. } => {
        (i18n("Processing Proofs").to_string(), theme_color().separator_color)
      }
      _ => {
        (i18n("Not Synced").to_string(), theme_color().separator_color)
      }
    }
  } else {
    (i18n("Not Connected").to_string(), theme_color().separator_color)
  }
}

pub fn connection_icon(state: NodeState) -> ( String, Color32 ) {
  if state.is_connected() {
    match state.sync_state.unwrap() {
      SyncState::UtxoResync { .. } |
      SyncState::Synced => {
        (egui_phosphor::bold::WIFI_HIGH.to_string(), Color32::GREEN)
      }
      _ => {
        (egui_phosphor::bold::DOWNLOAD.to_string(), Color32::YELLOW)
      }
    }
  } else {
    (egui_phosphor::bold::WIFI_SLASH.to_string(), Color32::RED)
  }
}

pub fn describe_peers(state: NodeState) -> String {
  let peercount = state.peers().unwrap_or(0_usize);
  format!("{} {}", peercount, i18n("Peers"))
}

pub fn describe_daa(state: NodeState) -> String {
  let daa_score = state.current_daa_score().unwrap_or(0_u64);
  format!("{} {}", i18n("DAA:"), format_number(daa_score))
}