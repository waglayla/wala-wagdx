use crate::imports::*;
use crate::dx_manager::Service;
pub use futures::{future::FutureExt, select, Future};
use waglayla_wallet_core::api::*;
use waglayla_wallet_core::events::Events as CoreWalletEvents;
#[allow(unused_imports)]
use waglayla_wallet_core::rpc::{
  ConnectOptions, ConnectStrategy, NotificationMode, Rpc, RpcCtl, WrpcEncoding,
};
use waglayla_wrpc_client::Resolver;
use workflow_core::runtime;

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

const ENABLE_PREEMPTIVE_DISCONNECT: bool = true;

cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    #[cfg(not(target_arch = "wasm32"))]
    use waglayla_rpc_service::service::RpcCoreService;

    const LOG_BUFFER_LINES: usize = 4096;
    const LOG_BUFFER_MARGIN: usize = 128;

    pub mod config;
    pub use config::Config;
    pub mod daemon;

    #[async_trait]
    pub trait Waglaylad {
      async fn start(self : Arc<Self>, config : Config) -> Result<()>;
      async fn stop(self : Arc<Self>) -> Result<()>;
    }
  }
}


cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    #[derive(Debug, Clone)]
    pub enum WaglayladServiceEvents {
      #[cfg(not(target_arch = "wasm32"))]
      StartInternalAsDaemon { config: Config, network: Network },
      StartRemoteConnection { rpc_config : RpcConfig, network : Network },
      Stdout { line : String },
      Disable { network : Network },
      Exit,
    }
  } else {
    #[derive(Debug)]
    pub enum WaglayladServiceEvents {
      StartRemoteConnection { rpc_config : RpcConfig, network : Network },
      Disable { network : Network },
      Exit,
    }
  }
}

pub struct WaglaylaService {
  pub application_events: ApplicationEventsChannel,
  pub service_events: Channel<WaglayladServiceEvents>,
  pub task_ctl: Channel<()>,
  pub network: Mutex<Network>,
  pub wallet: Arc<dyn WalletApi>,
  pub services_start_instant: Mutex<Option<Instant>>,
  #[cfg(not(target_arch = "wasm32"))]
  pub waglaylad: Mutex<Option<Arc<dyn Waglaylad + Send + Sync + 'static>>>,
  pub log_file: Mutex<std::fs::File>,
  pub daemon_sender: Sender<DaemonMessage>,
  pub connect_on_startup: Option<NodeSettings>,
}

impl WaglaylaService {
  pub fn new(
    application_events: ApplicationEventsChannel,
    settings: &Settings, 
    daemon_sender: Sender<DaemonMessage>,
    wallet: Option<Arc<dyn WalletApi>>,
  ) -> Self {

    let log_file = OpenOptions::new()
      .create(true)
      .write(true)
      .append(true)
      .open("rusty-waglayla-service.log")
      .expect("Failed to open log file");

    let storage = CoreWallet::local_store().unwrap_or_else(|e| {
      panic!("Failed to open local store: {}", e);
    });

    let wallet = wallet.unwrap_or_else(|| {
      Arc::new(
        CoreWallet::try_with_rpc(None, storage, Some(Network::Mainnet.into()))
          .unwrap_or_else(|e| {
            panic!("Failed to create wallet instance: {}", e);
          }),
      )
    });

    if !settings.initialized {
      log_warn!("WaglaylaService::new(): Settings are not initialized");
    }

    Self {
      application_events,
      service_events: Channel::unbounded(),
      task_ctl: Channel::oneshot(),
      network: Mutex::new(Network::Mainnet),
      wallet,
      services_start_instant: Mutex::new(None),
      #[cfg(not(target_arch = "wasm32"))]
      waglaylad: Mutex::new(None),
      log_file: Mutex::new(log_file),
      daemon_sender,
      connect_on_startup: settings.initialized.then(|| settings.node.clone()),
    }
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub fn retain(&self, waglaylad: Arc<dyn Waglaylad + Send + Sync + 'static>) {
    self.waglaylad.lock().unwrap().replace(waglaylad);
  }

  async fn stop_daemon(&self) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
      let waglaylad = self.waglaylad.lock().unwrap().take();
      if let Some(waglaylad) = waglaylad {
        if let Err(err) = waglaylad.stop().await {
          log_error!("error shutting down waglaylad: {}", err);
        }
      }
    }
    Ok(())
  }

  async fn disconnect_rpc(&self) -> Result<()> {
    if let Some(wallet) = self.core_wallet() {
      if let Ok(wrpc_client) = wallet.rpc_api().clone().downcast_arc::<WaglaylaRpcClient>() {
        wrpc_client.disconnect().await?;
      } else {
        wallet.rpc_ctl().signal_close().await?;
      }
    }
    Ok(())
  }

  fn is_wrpc_client(&self) -> bool {
    if let Some(wallet) = self.core_wallet() {
      wallet.has_rpc()
        && wallet
          .rpc_api()
          .clone()
          .downcast_arc::<WaglaylaRpcClient>()
          .is_ok()
    } else {
      false
    }
  }

  pub async fn stop_all_services(&self) -> Result<()> {
    self.services_start_instant.lock().unwrap().take();

    if let Some(wallet) = self.core_wallet() {
      if !wallet.has_rpc() {
        self.stop_daemon().await?;
        return Ok(());
      }

      let preemptive_disconnect = ENABLE_PREEMPTIVE_DISCONNECT && self.is_wrpc_client();

      if preemptive_disconnect {
        self.disconnect_rpc().await?;
      }

      for service in crate::dx_manager::manager().services().into_iter() {
        let instant = Instant::now();
        service.clone().rpc_detach().await?;
        if instant.elapsed().as_millis() > 1_000 {
          log_warn!(
            "WARNING: rpc_detach() for '{}' took {} msec",
            service.name(),
            instant.elapsed().as_millis()
          );
        }
      }

      if !preemptive_disconnect {
        self.disconnect_rpc().await?;
      }

      wallet.stop().await.expect("Unable to stop wallet");
      wallet.bind_rpc(None).await?;
    } else {
      self.wallet().disconnect().await?;
    }

    self.stop_daemon().await?;
    Ok(())
  }

  pub async fn start_all_services(
    self: &Arc<Self>,
    rpc: Option<Rpc>,
    network: Network,
  ) -> Result<()> {
    self.services_start_instant
      .lock()
      .unwrap()
      .replace(Instant::now());

    *self.network.lock().unwrap() = network;

    if let (Some(rpc), Some(wallet)) = (rpc, self.core_wallet()) {
      let rpc_api = rpc.rpc_api().clone();

      wallet
        .set_network_id(&network.into())
        .expect("Can not change network id while the wallet is connected");

      wallet.bind_rpc(Some(rpc)).await.unwrap();
      wallet
        .start()
        .await
        .expect("Unable to start wallet service");

      for service in crate::dx_manager::manager().services().into_iter() {
        service.rpc_attach(&rpc_api).await?;
      }

      Ok(())
    } else {
      self.wallet()
        .connect_call(ConnectRequest {
          url: None,
          network_id: network.into(),
          // retry_on_error: true,
          // block_async_connect: false,
          // require_sync: false,
        })
        .await?;

      Ok(())
    }
  }

  pub async fn apply_node_settings(&self, node_settings: &NodeSettings) -> Result<()> {
    match WaglayladServiceEvents::from_node_settings(node_settings, None) {
      Ok(event) => {
        self.service_events
          .sender
          .try_send(event)
          .unwrap_or_else(|err| {
            log_error!("WaglayladService error: {}", err);
          });
      }
      Err(err) => {
        log_error!("WaglayladServiceEvents::try_from() error: {}", err);
      }
    }
    Ok(())
  }

  pub fn update_services(&self, node_settings: &NodeSettings, options: Option<RpcOptions>) {
    match WaglayladServiceEvents::from_node_settings(node_settings, options) {
      Ok(event) => {
        self.service_events
          .sender
          .try_send(event)
          .unwrap_or_else(|err| {
            log_error!("WaglayladService error: {}", err);
          });
      }
      Err(err) => {
        log_error!("WaglayladServiceEvents::try_from() error: {}", err);
      }
    }
  }

  pub async fn connect_all_services(&self) -> Result<()> {
    for service in crate::dx_manager::manager().services().into_iter() {
      service.rpc_connect().await?;
    }

    Ok(())
  }

  pub async fn disconnect_all_services(&self) -> Result<()> {
    for service in crate::dx_manager::manager().services().into_iter() {
      service.rpc_disconnect().await?;
    }

    Ok(())
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn update_storage(&self) {
    const STORAGE_UPDATE_DELAY: Duration = Duration::from_millis(3000);

    let options = StorageUpdateOptions::default()
      .if_not_present()
      .with_delay(STORAGE_UPDATE_DELAY);

    manager().update_storage(options);
  }

  async fn handle_event(self: &Arc<Self>, event: WaglayladServiceEvents) -> Result<bool> {
    match event {
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladServiceEvents::Stdout { line } => {
        let wallet = self.core_wallet().ok_or(Error::WalletIsNotLocal)?;
        if !wallet.utxo_processor().is_synced() {
          wallet
            .utxo_processor()
            .sync_proc()
            .handle_stdout(&line)
            .await?;
        }

        let log_message = format!("{}\n", line);

        let mut file = self.log_file.lock().unwrap();
        if let Err(e) = file.write_all(log_message.as_bytes()) {
          log_error!("Failed to write to log file: {}", e);
        }

        let _ = self.daemon_sender.try_send(DaemonMessage(line));
      }
      #[cfg(not(target_arch = "wasm32"))]
      WaglayladServiceEvents::StartInternalAsDaemon { config, network } => {
        self.stop_all_services().await?;

        let waglaylad = Arc::new(daemon::Daemon::new(None, &self.service_events));
        self.retain(waglaylad.clone());
        waglaylad.clone().start(config).await.unwrap();

        let rpc = Self::create_rpc_client(Some("127.0.0.1".to_string()), None)
          .expect("Waglaylad Service - unable to create wRPC client");
        self.start_all_services(Some(rpc), network).await?;
        self.connect_rpc_client().await?;

        self.update_storage();
      }
      WaglayladServiceEvents::StartRemoteConnection {
        rpc_config,
        network,
      } => {
        if runtime::is_chrome_extension() {
          self.stop_all_services().await?;

          self.start_all_services(None, network).await?;
          self.connect_rpc_client().await?;
        } else {
          self.stop_all_services().await?;

          let rpc = Self::create_rpc_client(rpc_config.url(), None)
            .expect("Waglaylad Service - unable to create wRPC client");
          self.start_all_services(Some(rpc), network).await?;
          self.connect_rpc_client().await?;
        }
      }
      WaglayladServiceEvents::Disable { network } => {
        self.stop_all_services().await?;
        if let Some(wallet) = self.core_wallet() {
          self.stop_all_services().await?;


          if wallet.is_open() {
              wallet.close().await.ok();
          }

          wallet.set_network_id(&network.into()).ok();
        } else if runtime::is_chrome_extension() {
          self.stop_all_services().await?;
          self.wallet().wallet_close().await.ok();
          self.wallet().change_network_id(network.into()).await.ok();
        }
      }
      WaglayladServiceEvents::Exit => {
        return Ok(true);
      }
    }
    Ok(false)
  }

  async fn handle_multiplexer(
    &self,
    event: Box<waglayla_wallet_core::events::Events>,
  ) -> Result<()> {
    match *event {
      CoreWalletEvents::DaaScoreChange { .. } => {}
      CoreWalletEvents::Connect { .. } => {
        self.connect_all_services().await?;
      }
      CoreWalletEvents::Disconnect { .. } => {
        self.disconnect_all_services().await?;
      }
      _ => {
        log_info!("wallet event: {:?}", event);
      }
    }
    self.application_events
      .sender
      .send(crate::events::Events::Wallet { event })
      .await
      .unwrap();

    Ok(())
  }

  fn core_wallet_notify(&self, event: waglayla_wallet_core::events::Events) -> Result<()> {
    self.application_events
      .sender
      .try_send(crate::events::Events::Wallet {
        event: Box::new(event),
      })?;
    // .try_send(Box::new(event))?;
    Ok(())
  }

  fn notify(&self, event: crate::events::Events) -> Result<()> {
    self.application_events.sender.try_send(event)?;
    Ok(())
  }

  pub fn create_rpc_client(url: Option<String>, resolver_urls: Option<Vec<Arc<String>>>) -> Result<Rpc> {
    let resolver_or_none = match url {
      Some(_) => None,
      None => {
        if resolver_urls.is_none() {
          Some(Resolver::default())
        } else {
          Some(Resolver::new(resolver_urls.clone().unwrap_or_else(|| Vec::new())))
        }
      }
    };

    let url = url.clone().unwrap_or_else(|| "127.0.0.1".to_string());
    let url =
      WaglaylaRpcClient::parse_url(url, WrpcEncoding::Borsh, NetworkId::from(Network::Mainnet).into())?;

    log_info!("using url {}", url);

    let wrpc_client = Arc::new(WaglaylaRpcClient::new_with_args(
      WrpcEncoding::Borsh,
      if resolver_or_none.is_some() {
        None
      } else {
        Some(url.as_str())
      },
      resolver_or_none,
      Some(NetworkId::from(Network::Mainnet)),
      None,
    )?);
    let rpc_ctl = wrpc_client.ctl().clone();
    let rpc_api: Arc<DynRpcApi> = wrpc_client;
    Ok(Rpc::new(rpc_api, rpc_ctl))
  }

  pub fn wallet(&self) -> Arc<dyn WalletApi> {
    self.wallet.clone()
  }

  pub fn core_wallet(&self) -> Option<Arc<CoreWallet>> {
    self.wallet.clone().downcast_arc::<CoreWallet>().ok()
  }

  pub async fn connect_rpc_client(&self) -> Result<()> {
    if let Some(wallet) = self.core_wallet() {
      if let Ok(wrpc_client) = wallet.rpc_api().clone().downcast_arc::<WaglaylaRpcClient>() {
        let options = ConnectOptions {
          block_async_connect: false,
          strategy: ConnectStrategy::Retry,
          url: None,
          connect_timeout: None,
          retry_interval: Some(Duration::from_millis(3000)),
        };
        wrpc_client.connect(Some(options)).await?;
      } else {
        #[cfg(not(target_arch = "wasm32"))]
        {
          if wallet
            .rpc_api()
            .clone()
            .downcast_arc::<RpcCoreService>()
            .is_ok()
          {
            wallet.rpc_ctl().signal_open().await?;
          } else {
            unimplemented!("connect_rpc_client(): RPC client is not supported")
          }
        }
      }
    }
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Context {}

#[async_trait]
impl Service for WaglaylaService {
  fn name(&self) -> &'static str {
    "waglayla-service"
  }

  async fn launch(self: Arc<Self>) -> Result<()> {
    let status = None;

    if let Some(status) = status {
      let GetStatusResponse {
        is_connected,
        is_open: _,
        is_synced,
        url,
        is_wrpc_client: _,
        network_id,
        context,
        selected_account_id,
        wallet_descriptor,
        account_descriptors,
      } = status;

      if let Some(context) = context {
        let _context = Context::try_from_slice(&context)?;

        if is_connected {
          let network_id = network_id.unwrap_or_else(|| Network::Mainnet.into());

          // let event = Box::new(waglayla_wallet_core::events::Events::Connect {
          //     network_id,
          //     url: url.clone(),
          // });
          // self.application_events
          //     .sender
          //     .try_send(crate::events::Events::Wallet { event })
          //     // .await
          //     .unwrap();

          self.core_wallet_notify(CoreWalletEvents::Connect {
            network_id,
            url: url.clone(),
          })
          .unwrap();

          // ^ TODO - Get appropriate `server_version`
          let server_version = Default::default();
          // let event = Box::new(CoreWalletEvents::ServerStatus {
          //     is_synced,
          //     network_id,
          //     url,
          //     server_version,
          // });
          // self.application_events
          //     .sender
          //     .try_send(crate::events::Events::Wallet { event })
          //     // .await
          //     .unwrap();

          self.core_wallet_notify(CoreWalletEvents::ServerStatus {
            is_synced,
            network_id,
            url,
            server_version,
          })
          .unwrap();
        }

        if let (Some(wallet_descriptor), Some(account_descriptors)) =
          (wallet_descriptor, account_descriptors)
        {
          self.core_wallet_notify(CoreWalletEvents::WalletOpen {
            wallet_descriptor: Some(wallet_descriptor),
            account_descriptors: Some(account_descriptors),
          })
          .unwrap();
        }

        if let Some(selected_account_id) = selected_account_id {
          self.core_wallet_notify(CoreWalletEvents::AccountSelection {
            id: Some(selected_account_id),
          })
          .unwrap();

          // self.notify(crate::events::Events::ChangeSection(TypeId::of::<
          //     crate::modules::account_manager::AccountManager,
          // >(
          // )))
          // .unwrap();
        }
      } else {
        // new instance - emit startup event
        if let Some(node_settings) = self.connect_on_startup.as_ref() {
          self.apply_node_settings(node_settings).await?;
        }

        // new instance - setup new context
        // let context = Context {};
        // self.wallet()
        //     .retain_context("kaspa-ng", Some(borsh::to_vec(&context)?))
        //     .await?;
      }
    } else {
      if let Some(node_settings) = self.connect_on_startup.as_ref() {
        self.apply_node_settings(node_settings).await?;
      }
    }

    if let Some(wallet) = self.core_wallet() {
      // wallet.multiplexer().channel()
      let wallet_events = wallet.multiplexer().channel();

      loop {
        select! {
          msg = wallet_events.recv().fuse() => {
          // msg = wallet.multiplexer().channel().recv().fuse() => {
            if let Ok(event) = msg {
              self.handle_multiplexer(event).await?;
            } else {
              break;
            }
          }

          msg = self.as_ref().service_events.receiver.recv().fuse() => {
            if let Ok(event) = msg {
              if self.handle_event(event).await? {
                break;
              }

            } else {
              break;
            }
          }
        }
      }
    } else {
      loop {
        select! {
          msg = self.as_ref().service_events.receiver.recv().fuse() => {
            if let Ok(event) = msg {
              if self.handle_event(event).await? {
                break;
              }

            } else {
              break;
            }
          }
        }
      }
    };

    self.stop_all_services().await?;
    self.task_ctl.send(()).await.unwrap();

    Ok(())
  }

  fn terminate(self: Arc<Self>) {
    self.service_events
      .sender
      .try_send(WaglayladServiceEvents::Exit)
      .unwrap();
  }

  async fn join(self: Arc<Self>) -> Result<()> {
    self.task_ctl.recv().await.unwrap();
    Ok(())
  }
}

impl WaglayladServiceEvents {
  pub fn from_node_settings(
      node_settings: &NodeSettings,
      options: Option<RpcOptions>,
  ) -> Result<Self> {
    cfg_if! {
      if #[cfg(not(target_arch = "wasm32"))] {
        match &node_settings.node_kind {
          WaglayladNodeKind::Disabled => {
            Ok(WaglayladServiceEvents::Disable { network : Network::Mainnet })
          }
          WaglayladNodeKind::IntegratedAsDaemon => {
            Ok(WaglayladServiceEvents::StartInternalAsDaemon { config : Config::from(node_settings.clone()), network : Network::Mainnet })
          }
          WaglayladNodeKind::Remote => {
            Ok(WaglayladServiceEvents::StartRemoteConnection { rpc_config : RpcConfig::from_node_settings(node_settings,options), network : Network::Mainnet })
          }
        }
      } else {
        match &node_settings.node_kind {
          WaglayladNodeKind::Remote => {
            Ok(WaglayladServiceEvents::StartRemoteConnection { rpc_config : RpcConfig::from_node_settings(node_settings,options), network : Network::Mainnet })
          }
        }
      }
    }
  }
}
