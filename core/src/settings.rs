use crate::imports::*;
use waglayla_metrics_core::Metric;
use waglayla_utils::networking::ContextualNetAddress;
use waglayla_wallet_core::storage::local::storage::Storage;
use waglayla_wrpc_client::WrpcEncoding;
use workflow_core::{runtime, task::spawn};

use sys_locale::get_locale;
use serde_json::Value;

const SETTINGS_REVISION: &str = "0.0.0";

// Node endpoint location settings
cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(rename_all = "kebab-case")]
    pub enum WaglayladNodeKind {
      Disabled,
      Remote,
      #[default]
      IntegratedAsDaemon,
    }

    const WAGLAYLAD_NODE_KINDS: [WaglayladNodeKind; 3] = [
      WaglayladNodeKind::Disabled,
      WaglayladNodeKind::Remote,
      WaglayladNodeKind::IntegratedAsDaemon,
    ];

    impl std::fmt::Display for WaglayladNodeKind {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          WaglayladNodeKind::Disabled => write!(f, "{}", i18n("Disabled")),
          WaglayladNodeKind::Remote => write!(f, "{}", i18n("Remote")),
          WaglayladNodeKind::IntegratedAsDaemon => write!(f, "{}", i18n("Integrated Daemon")),
        }
      }
    }

  } else {
    #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
    #[serde(rename_all = "kebab-case")]
    pub enum WaglayladNodeKind {
      #[default]
      Remote,
    }

    const WAGLAYLAD_NODE_KINDS: [WaglayladNodeKind; 1] = [
      WaglayladNodeKind::Remote,
    ];

    impl std::fmt::Display for WaglayladNodeKind {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
          WaglayladNodeKind::Remote => write!(f, "Remote"),
        }
      }
    }
  }
}

impl WaglayladNodeKind {
  pub fn iter() -> impl Iterator<Item = &'static WaglayladNodeKind> {
    WAGLAYLAD_NODE_KINDS.iter()
  }

  pub fn describe(&self) -> &str {
    match self {
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladNodeKind::Disabled => i18n("Disables waglaylad. Required for deleting or changing the storage location. (These can be done in the Settings menu after startup)"),
      WaglayladNodeKind::Remote => i18n("Connects to a Remote Rusty Waglayla Node via wRPC."),
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladNodeKind::IntegratedAsDaemon => i18n("The node is spawned as a child daemon process (recommended)."),
    }
  }

  pub fn is_config_capable(&self) -> bool {
    match self {
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladNodeKind::Disabled => false,
      WaglayladNodeKind::Remote => false,
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladNodeKind::IntegratedAsDaemon => true,
    }
  }

  pub fn is_local(&self) -> bool {
    match self {
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladNodeKind::Disabled => false,
      WaglayladNodeKind::Remote => false,
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladNodeKind::IntegratedAsDaemon => true,
    }
  }
}


// RPC configuration parameters for gRPC
#[derive(Default)]
pub struct RpcOptions {
  pub blacklist_servers: Vec<String>,
}

impl RpcOptions {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn blacklist(mut self, server: String) -> Self {
    self.blacklist_servers.push(server);
    self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RpcConfig {
  // Wrpc is unnecessary here
  Wrpc {
    url: Option<String>,
    encoding: WrpcEncoding,
    resolver_urls: Option<Vec<Arc<String>>>,
  },
}

impl Default for RpcConfig {
  fn default() -> Self {
    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            let url = "127.0.0.1";
        } else {
            use workflow_dom::utils::*;
            let url = window().location().hostname().expect("KaspadNodeKind: Unable to get hostname");
        }
    }
    RpcConfig::Wrpc {
      url: Some(url.to_string()),
      encoding: WrpcEncoding::Borsh,
      resolver_urls: None,
    }
  }
}

impl RpcConfig {
  pub fn url(&self) -> Option<String> {
      match self {
          RpcConfig::Wrpc { url, .. } => url.clone(),
      }
  }

  pub fn encoding(&self) -> Option<WrpcEncoding> {
      match self {
          RpcConfig::Wrpc { encoding, .. } => Some(*encoding),
      }
  }

  pub fn resolver_urls(&self) -> Option<Vec<Arc<String>>> {
      match self {
          RpcConfig::Wrpc { resolver_urls, .. } => resolver_urls.clone(),
      }
  }
}

// Endpoint interface parameters
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NetworkInterfaceKind {
  #[default]
  Local,
  Any,
  Custom,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NetworkInterfaceConfig {
  #[serde(rename = "type")]
  pub kind: NetworkInterfaceKind,
  pub custom: ContextualNetAddress,
}

impl Default for NetworkInterfaceConfig {
  fn default() -> Self {
    Self {
      kind: NetworkInterfaceKind::Local,
      custom: ContextualNetAddress::loopback(),
    }
  }
}

impl From<NetworkInterfaceConfig> for ContextualNetAddress {
  fn from(network_interface_config: NetworkInterfaceConfig) -> Self {
    match network_interface_config.kind {
      NetworkInterfaceKind::Local => "127.0.0.1".parse().unwrap(),
      NetworkInterfaceKind::Any => "0.0.0.0".parse().unwrap(),
      NetworkInterfaceKind::Custom => network_interface_config.custom,
    }
  }
}

impl std::fmt::Display for NetworkInterfaceConfig {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ContextualNetAddress::from(self.clone()).fmt(f)
  }
}

// 
#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum NodeConnectionConfigKind {
  #[default]
  PublicServerRandom,
  PublicServerCustom,
  Custom,
  // Local,
}

impl std::fmt::Display for NodeConnectionConfigKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NodeConnectionConfigKind::PublicServerRandom => {
        write!(f, "{}", i18n("Random Public Node"))
      }
      NodeConnectionConfigKind::PublicServerCustom => {
        write!(f, "{}", i18n("Custom Public Node"))
      }
      NodeConnectionConfigKind::Custom => write!(f, "{}", i18n("Custom")),
      // NodeConnectionConfigKind::Local => write!(f, "{}", i18n("Local")),
    }
  }
}

impl NodeConnectionConfigKind {
  pub fn iter() -> impl Iterator<Item = &'static NodeConnectionConfigKind> {
    [
      NodeConnectionConfigKind::PublicServerRandom,
      // NodeConnectionConfigKind::PublicServerCustom,
      NodeConnectionConfigKind::Custom,
      // NodeConnectionConfigKind::Local,
    ]
    .iter()
  }

  pub fn is_public(&self) -> bool {
    matches!(
      self,
      NodeConnectionConfigKind::PublicServerRandom
        | NodeConnectionConfigKind::PublicServerCustom
    )
  }
}

// Leave the management to the Waglayla daemon
pub const NODE_MEMORY_SCALE: f64 = 1.0;

// Complete settings suite/section for the daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeSettings {
  pub connection_config_kind: NodeConnectionConfigKind,
  pub wrpc_url: String,
  #[serde(default)]
  pub enable_wrpc_borsh: bool,
  #[serde(default)]
  pub wrpc_borsh_network_interface: NetworkInterfaceConfig,
  pub wrpc_encoding: WrpcEncoding,
  pub enable_wrpc_json: bool,
  pub wrpc_json_network_interface: NetworkInterfaceConfig,
  pub enable_grpc: bool,
  pub grpc_network_interface: NetworkInterfaceConfig,
  pub enable_upnp: bool,

  // pub network: Network,
  pub node_kind: WaglayladNodeKind,
  pub waglaylad_daemon_binary: String,
  pub waglaylad_daemon_args: String,
  pub waglaylad_daemon_args_enable: bool,
  #[serde(default)]
  pub waglaylad_daemon_storage_folder_enable: bool,
  #[serde(default)]
  pub waglaylad_daemon_storage_folder: String,
}

impl Default for NodeSettings {
  fn default() -> Self {
    Self {
      connection_config_kind: NodeConnectionConfigKind::default(),
      wrpc_url: "127.0.0.1".to_string(),
      wrpc_encoding: WrpcEncoding::Borsh,
      enable_wrpc_borsh: false,
      wrpc_borsh_network_interface: NetworkInterfaceConfig::default(),
      enable_wrpc_json: false,
      wrpc_json_network_interface: NetworkInterfaceConfig::default(),
      enable_grpc: true,
      grpc_network_interface: NetworkInterfaceConfig::default(),
      enable_upnp: true,
      // network: Network::default(),
      node_kind: WaglayladNodeKind::default(),
      waglaylad_daemon_binary: String::default(),
      waglaylad_daemon_args: String::default(),
      waglaylad_daemon_args_enable: false,
      waglaylad_daemon_storage_folder_enable: false,
      waglaylad_daemon_storage_folder: String::default(),
    }
  }
}

impl NodeSettings {
  cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
      #[allow(clippy::if_same_then_else)]
      pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
        if self.node_kind != other.node_kind {
          Some(true)
        } else if self.connection_config_kind != other.connection_config_kind
        {
          Some(true)
        } else if self.waglaylad_daemon_storage_folder_enable != other.waglaylad_daemon_storage_folder_enable
          || other.waglaylad_daemon_storage_folder_enable && (self.waglaylad_daemon_storage_folder != other.waglaylad_daemon_storage_folder)
        {
          Some(true)
        } else if self.enable_grpc != other.enable_grpc
          || self.grpc_network_interface != other.grpc_network_interface
          || self.wrpc_url != other.wrpc_url
          || self.wrpc_encoding != other.wrpc_encoding
          || self.enable_wrpc_json != other.enable_wrpc_json
          || self.wrpc_json_network_interface != other.wrpc_json_network_interface
          || self.enable_upnp != other.enable_upnp
        {
          Some(true)
        } else if self.waglaylad_daemon_args != other.waglaylad_daemon_args
          || self.waglaylad_daemon_args_enable != other.waglaylad_daemon_args_enable
        {
          Some(self.node_kind.is_config_capable())
        } else {
          None
        }
      }
    } else {
      #[allow(clippy::if_same_then_else)]
      pub fn compare(&self, other: &NodeSettings) -> Option<bool> {
        let basic_diff = 
        self.network != other.network ||
        self.node_kind != other.node_kind ||
        self.connection_config_kind != other.connection_config_kind;

        if basic_diff {
            return Some(true);
        } else if self.wrpc_url != other.wrpc_url
          || self.wrpc_encoding != other.wrpc_encoding
        {
          Some(true)
        } else {
          None
        }
      }
    }
  }
}

// Complete settings suite/section for the RPC setup
impl RpcConfig {
  pub fn from_node_settings(settings: &NodeSettings, _options: Option<RpcOptions>) -> Self {
    RpcConfig::Wrpc {
      url: Some(settings.wrpc_url.clone()),
      encoding: settings.wrpc_encoding,
      resolver_urls: None,
    }
  }
}

// Metrics display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct MetricsSettings {
  pub graph_columns: usize,
  pub graph_height: usize,
  pub graph_range_from: isize,
  pub graph_range_to: isize,
  pub disabled: AHashSet<Metric>,
}

impl Default for MetricsSettings {
  fn default() -> Self {
    Self {
      graph_columns: 3,
      graph_height: 90,
      graph_range_from: -15 * 60,
      graph_range_to: 0,
      disabled: AHashSet::default(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserInterfaceSettings {
  pub theme_color: String,
  pub theme_style: String,
  pub scale: f32,
  pub metrics: MetricsSettings,
  pub balance_padding: bool,
  // #[serde(default)]
  // pub disable_frame: bool,
}

// Assembled ensemble of all settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
  pub revision: String,
  pub initialized: bool,
  pub splash_screen: bool,
  pub compound_info_screen: bool,
  pub version: String,
  pub update: String,
  #[serde(default)]
  pub node: NodeSettings,
  pub user_interface: UserInterfaceSettings,
  pub language_code: String,
  pub update_monitor: bool,
  pub market_monitor: bool,
  // #[serde(default)]
  // pub disable_frame: bool,
}

impl Default for UserInterfaceSettings {
  fn default() -> Self {
    // cfg_if! {
    //     if #[cfg(target_os = "windows")] {
    //         let disable_frame = true;
    //     } else {
    //         let disable_frame = false;
    //     }
    // }

    Self {
      theme_color: "Dark".to_string(),
      theme_style: "Rounded".to_string(),
      scale: 1.0,
      metrics: MetricsSettings::default(),
      balance_padding: true,
    }
  }
}

impl Default for Settings {
  fn default() -> Self {
    let system_language = get_locale().unwrap_or_else(|| "en".to_string());
    let base_system_language = system_language.split('-').next().map(|s| s.to_string()).unwrap_or(system_language);

    // Parse the embedded JSON data
    let translations: Value = serde_json::from_str(include_str!("../resources/i18n/i18n.json"))
        .expect("Embedded workflow_i18n.json is invalid");

    let language_code = if translations.get("translations").unwrap().get(&base_system_language).is_some() {
      base_system_language.clone()
    } else {
      "en".to_string()
    };

    Self {
      initialized: false,
      revision: SETTINGS_REVISION.to_string(),

      splash_screen: true,
      compound_info_screen: true,
      version: "0.0.0".to_string(),
      update: crate::app::VERSION.to_string(),
      node: NodeSettings::default(),
      user_interface: UserInterfaceSettings::default(),
      language_code,
      update_monitor: true,
      market_monitor: true,
      // disable_frame: false,
    }
  }
}

impl Settings {}

fn storage() -> Result<Storage> {
  Ok(Storage::try_new("wala-wagdx.settings")?)
}

impl Settings {
  pub async fn store(&self) -> Result<()> {
    let storage = storage()?;
    storage.ensure_dir().await?;
    workflow_store::fs::write_json(storage.filename(), self).await?;
    Ok(())
  }

  pub fn store_sync(&self) -> Result<&Self> {
    let storage = storage()?;
    if runtime::is_chrome_extension() {
      let this = self.clone();
      spawn(async move {
        println!("{}", storage.filename().display());
        if let Err(err) = workflow_store::fs::write_json(storage.filename(), &this).await {
          log_error!("Settings::store_sync() error: {}", err);
        }
      });
    } else {
      storage.ensure_dir_sync()?;
      workflow_store::fs::write_json_sync(storage.filename(), self)?;
    }
    Ok(self)
  }

  pub async fn load() -> Result<Self> {
    use workflow_store::fs::read_json;

    let storage = storage()?;
    if storage.exists().await.unwrap_or(false) {
      match read_json::<Self>(storage.filename()).await {
        Ok(mut settings) => {
          if settings.revision != SETTINGS_REVISION {
            Ok(Self::default())
          } else {
            if matches!(
              settings.node.connection_config_kind,
              NodeConnectionConfigKind::PublicServerCustom
            ) {
              settings.node.connection_config_kind =
                NodeConnectionConfigKind::PublicServerRandom;
            }

            Ok(settings)
          }
        }
        Err(error) => {
          #[allow(clippy::if_same_then_else)]
          if matches!(error, workflow_store::error::Error::SerdeJson(..)) {
            // TODO - recovery process
            log_warn!("Settings::load() error: {}", error);
            Ok(Self::default())
          } else {
            log_warn!("Settings::load() error: {}", error);
            Ok(Self::default())
          }
        }
      }
    } else {
      Ok(Self::default())
    }
  }
}
