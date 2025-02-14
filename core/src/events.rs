use crate::imports::*;
// use crate::market::*;
use crate::storage::StorageUpdateOptions;
use crate::core::ToastKind;
// use crate::utils::Release;
use waglayla_metrics_core::MetricsSnapshot;
use waglayla_wallet_core::{events as waglayla, storage::PrvKeyDataInfo};

pub type ApplicationEventsChannel = crate::dx_manager::channel::Channel<Events>;

#[derive(Clone, Debug)]
pub enum Events {
  ChangeSection(TypeId),
  NetworkChange(Network),
  UpdateStorage(StorageUpdateOptions),
  VisibilityChange(VisibilityState),
  PeerCountUpdate(usize),
  BlockRewardUpdate(u64),
  CoinSupplyUpdate(u64, u64),
  HashrateUpdate(u64),
  DifficultyUpdate(u64),
  MempoolUpdate(usize),
  Notify(&'static str, ToastKind, u64),
  // VersionUpdate(Release),
  ThemeChange,
  StoreSettings,
  UpdateLogs,
  // Market(MarketUpdate),
  Metrics {
    snapshot: Box<MetricsSnapshot>,
  },
  MempoolSize {
    mempool_size: usize,
  },
  Error(Box<String>),
  WalletList {
    wallet_list: Arc<Vec<WalletDescriptor>>,
  },
  Wallet {
    event: Box<waglayla::Events>,
  },
  WalletUpdate,
  PrvKeyDataInfo {
    prv_key_data_info_map: HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>,
  },
  UnlockSuccess,
  UnlockFailure {
    message: String,
  },
  NodeInfo {
    node_info: Option<Box<String>>,
  },
  Close,
  Exit,
}
