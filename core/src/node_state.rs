use waglayla_consensus_core::network::NetworkId;
use waglayla_metrics_core::MetricsSnapshot;
use waglayla_wallet_core::events::SyncState;

#[derive(Default, Clone)]
pub struct NodeState {
  pub is_open: bool,
  pub is_connected: bool,
  pub is_synced: Option<bool>,
  pub sync_state: Option<SyncState>,
  pub server_version: Option<String>,
  pub url: Option<String>,
  pub network_id: Option<NetworkId>,
  pub current_daa_score: Option<u64>,
  pub node_metrics: Option<Box<MetricsSnapshot>>,
  pub node_peers: Option<usize>,
  pub node_mempool_size: Option<usize>,
  pub network_tps: Option<f64>,

  pub block_reward: Option<u64>,
  pub current_supply: Option<u64>,
  pub max_supply: Option<u64>,
  pub hashes_per_second: Option<u64>,
  pub difficulty: Option<u64>,

  pub error: Option<String>,
}

impl NodeState {
  pub fn is_open(&self) -> bool {
    self.is_open
  }

  pub fn is_connected(&self) -> bool {
    self.is_connected
  }

  pub fn is_synced(&self) -> bool {
    self.is_synced.unwrap_or(false) || matches!(self.sync_state, Some(SyncState::Synced))
  }

  pub fn sync_state(&self) -> &Option<SyncState> {
    &self.sync_state
  }

  pub fn server_version(&self) -> &Option<String> {
    &self.server_version
  }

  pub fn url(&self) -> &Option<String> {
    &self.url
  }

  pub fn network_id(&self) -> &Option<NetworkId> {
    &self.network_id
  }

  pub fn current_daa_score(&self) -> Option<u64> {
    self.current_daa_score
  }

  pub fn error(&self) -> &Option<String> {
    &self.error
  }

  pub fn metrics(&self) -> Option<&MetricsSnapshot> {
    self.node_metrics.as_deref()
  }

  pub fn peers(&self) -> Option<usize> {
    self.node_peers
      .or_else(|| self.metrics().map(|m| m.data.node_active_peers as usize))
  }

  pub fn tps(&self) -> Option<f64> {
    self.network_tps
      .or_else(|| self.metrics().map(|m| m.network_transactions_per_second))
  }

  pub fn block_reward(&self) -> Option<u64> {
    self.block_reward
  }

  pub fn current_supply(&self) -> Option<u64> {
    self.current_supply
  }

  pub fn max_supply(&self) -> Option<u64> {
    self.max_supply
  }

  pub fn hashes_per_second(&self) -> Option<u64> {
    self.hashes_per_second
  }

  pub fn difficulty(&self) -> Option<u64> {
    self.difficulty
  }

  pub fn mempool_size(&self) -> Option<usize> {
    self.node_mempool_size
  }
}
