pub use cfg_if::cfg_if;
pub use downcast_rs::{impl_downcast, Downcast, DowncastSync};
pub use waglayla_consensus_core::constants::SOMPI_PER_WAGLAYLA;
pub use waglayla_consensus_core::network::{NetworkId, NetworkType};
pub use waglayla_consensus_core::Hash as WaglaylaHash;
pub use waglayla_metrics_core::MetricsSnapshot;
pub use waglayla_rpc_core::api::rpc::RpcApi;
// pub use waglayla_rpc_core::{RpcFeeEstimate, RpcFeerateBucket}; TODO maybe
pub use waglayla_utils::hex::{FromHex, ToHex};
pub use waglayla_utils::{hashmap::GroupExtension, networking::ContextualNetAddress};
pub use waglayla_wallet_core::prelude::{
  Account as CoreAccount, AccountCreateArgs, AccountCreateArgsBip32, AccountDescriptor,
  AccountId, AccountKind, Address, Balance, DynRpcApi, IdT, WaglaylaRpcClient, Language,
  MetricsUpdate, MetricsUpdateKind, Mnemonic, PrvKeyDataArgs, PrvKeyDataCreateArgs, PrvKeyDataId,
  PrvKeyDataInfo, Secret, SyncState, TransactionId, TransactionRecord, Wallet as CoreWallet,
  WalletApi, WalletCreateArgs, WalletDescriptor, WordCount, WrpcEncoding,
};
pub use waglayla_wallet_core::utils::*;

pub use async_trait::async_trait;
pub use borsh::{BorshDeserialize, BorshSerialize};
pub use futures::{pin_mut, select, FutureExt, StreamExt};
pub use futures_util::future::{join_all, try_join_all};
pub use separator::*;
pub use serde::{Deserialize, Serialize};
pub use std::any::{Any, TypeId};
pub use std::cell::{Ref, RefCell, RefMut};
pub use std::collections::HashMap;
pub use std::collections::VecDeque;
pub use std::future::Future;
pub use std::path::{Path, PathBuf};
pub use std::pin::Pin;
pub use std::rc::Rc;
pub use std::str::FromStr;
pub use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
pub use std::sync::OnceLock;
pub use std::sync::{Arc, Mutex, MutexGuard, RwLock};
pub use std::time::Duration;

pub use web_sys::VisibilityState;
pub use workflow_core::abortable::Abortable;
pub use workflow_core::channel::{oneshot, Channel, Receiver, Sender};
pub use workflow_core::enums::Describe;
pub use workflow_core::extensions::is_not_empty::*;
pub use workflow_core::task;
pub use workflow_core::task::{sleep, yield_executor};
pub use workflow_core::time::{unixtime_as_millis_f64, Instant};
pub use workflow_dom::utils::*;
pub use workflow_http as http;
pub use workflow_i18n::i18n_args;
pub use workflow_i18n::prelude::*;
pub use workflow_log::prelude::*;

pub use ahash::{AHashMap, AHashSet};
pub use pad::{Alignment, PadStr};
pub use rand::Rng;
pub use slug::slugify;
pub use zeroize::*;

pub use egui::epaint::{
  text::{LayoutJob, TextFormat},
  FontFamily, FontId,
};
pub use egui::*;
pub use egui_plot::{PlotPoint, PlotPoints};

pub use crate::collection::Collection;
pub use crate::core::Core;
pub use crate::core::MAINNET_EXPLORER;
pub use crate::core::TESTNET10_EXPLORER;
pub use crate::core::TESTNET11_EXPLORER;

pub use crate::fonts::get_font_family;

// pub use crate::device::{Device, Orientation};
pub use crate::gui::*;
pub use crate::error::Error;
pub use crate::events::{ApplicationEventsChannel, Events};
// pub use crate::extensions::*;
// pub use crate::interop;
// pub use crate::market::MarketData;
// pub use crate::menu::Menu;
pub use crate::components;
pub use crate::components::*;
pub use crate::network::BASIC_TRANSACTION_MASS;
pub use crate::network::{Network, NetworkPressure};
pub use crate::gui::widgets::*;

pub use crate::frame::*;
// pub use crate::notifications::{Notifications, UserNotification, UserNotifyKind};
// pub use crate::primitives::{
//     Account, AccountCollection, AccountSelectorButtonExtension, BlockDagGraphSettings, DaaBucket,
//     DagBlock, Transaction, TransactionCollection,
// };
pub use crate::dx_wallet::*;
pub use crate::result::Result;
// pub use crate::runtime::{runtime, spawn, spawn_with_result, Payload, Runtime, Service};
pub use crate::dx_manager::{manager, DX_Manager, Service};
pub use crate::dx_manager::channel::{ DaemonMessage };

pub use crate::settings::{
  WaglayladNodeKind, NetworkInterfaceConfig, NetworkInterfaceKind,
  NodeConnectionConfigKind, NodeSettings, RpcConfig, RpcOptions, Settings,
  UserInterfaceSettings,
};
pub use crate::node_state::NodeState;
// pub use crate::status::Status;
pub use crate::storage::{Storage, StorageUpdateOptions};
pub use crate::utils::*;

pub use strum::IntoEnumIterator;
pub use strum_macros::{EnumIter, Display};