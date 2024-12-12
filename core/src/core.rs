use crate::frame::window_frame;
use crate::imports::*;

use crate::components::HashMapComponentExtension;
use crate::components::outline::Outline;
use crate::components::hello::Hello;
use crate::components::blank::Blank;
use crate::components::console::DaemonConsole;
use crate::components::welcome::Welcome;
use crate::components::footer::Footer;

use crate::components::wallet_ui::*;

// use crate::market::*; TODO: make our own market monitoring solution
// use crate::mobile::MobileMenu; TODO: make own version of this
use egui::load::Bytes;
use egui_notify::Toasts;
use waglayla_wallet_core::api::TransactionsDataGetResponse;
use waglayla_wallet_core::events::Events as CoreWallet;
use waglayla_wallet_core::storage::{Binding, Hint, PrvKeyDataInfo};
use std::borrow::Cow;
use std::future::IntoFuture;
#[allow(unused_imports)]
use workflow_i18n::*;
use workflow_wasm::callback::CallbackMap;
pub const TRANSACTION_PAGE_SIZE: u64 = 20;
pub const MAINNET_EXPLORER: &str = "https://explorer.waglayla.org";
pub const TESTNET10_EXPLORER: &str = "https://explorer-tn10.waglayla.org";
pub const TESTNET11_EXPLORER: &str = "https://explorer-tn11.waglayla.org";

use eframe::egui::{self, Context, Ui, RichText, Color32, Stroke};

#[derive(Clone)]
pub enum Exception {
  #[allow(dead_code)]
  UtxoIndexNotEnabled { url: Option<String> },
}

#[derive(Clone)]
pub struct Core {
  is_shutdown_pending: bool,
  account_updated: bool,
  settings_storage_requested: bool,
  last_settings_storage_request: Instant,
  manager: DX_Manager,
  wallet: Arc<dyn WalletApi>,
  application_events_channel: ApplicationEventsChannel,
  // deactivation: Option<Module>,
  prev_components: Option<Vec<TypeId>>,
  component: Component,
  components: HashMap<TypeId, Component>,
  pub stack: VecDeque<Component>,
  pub settings: Settings,
  // pub toasts: Toasts,
  pub mobile_style: egui::Style,
  pub default_style: egui::Style,

  node_state: NodeState,
  hint: Option<Hint>,
  discard_hint: bool,
  exception: Option<Exception>,
  // screenshot: Option<Arc<ColorImage>>,

  pub wallet_descriptor: Option<WalletDescriptor>,
  pub wallet_list: Vec<WalletDescriptor>,
  pub prv_key_data_map: Option<HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>>,
  pub user_accounts: Option<AccountGroup>,
  pub current_account: Option<Account>,
  // pub release: Option<Release>,

  // pub device: Device,
  // pub market: Option<Market>,
  // pub debug: bool,
  pub window_frame: bool,
  // callback_map: CallbackMap,
  // pub network_pressure: NetworkPressure,
  // notifications: Notifications,
  pub storage: Storage,
  // pub feerate : Option<Arc<RpcFeeEstimate>>,
  // pub feerate: Option<FeerateEstimate>, TODO maybe
  pub node_info: Option<Box<String>>,
  daemon_console: Component,
  footer: Component,
}

impl Core {
  pub fn new(
    cc: &eframe::CreationContext<'_>,
    settings: Settings,
    window_frame: bool,
    daemon_receiver: Receiver<DaemonMessage>,
  ) -> Self {
    // Initialize fonts if needed
    crate::fonts::init_fonts(cc);
    
    // Create default styles
    let mut default_style = (*cc.egui_ctx.style()).clone();
    let mut mobile_style = (*cc.egui_ctx.style()).clone();

    let manager = manager();
    
    // Apply your theme
    apply_theme_by_name(
      &cc.egui_ctx,
      settings.user_interface.theme_color.as_str(),
      settings.user_interface.theme_style.as_str(),
    );

    let mut components = HashMap::new();
    components.insert_typeid(Welcome::new(manager.clone()));
    components.insert_typeid(Outline::new(&cc.egui_ctx));
    components.insert_typeid(Hello::default());
    components.insert_typeid(Blank::default());
    components.insert_typeid(components::settings::Settings::new(manager.clone()));
    components.insert_typeid(CreateWallet::new(manager.clone()));
    components.insert_typeid(OpenWallet::new(manager.clone()));
    components.insert_typeid(DaemonConsole::new(daemon_receiver));
    components.insert_typeid(ViewWallet::new(manager.clone()));
    components.insert_typeid(WalletDelegator::default());

    components.insert_typeid(Footer::default());
    let footer = components.get(&TypeId::of::<Footer>()).unwrap().clone();

    let storage = Storage::default();
    #[cfg(not(target_arch = "wasm32"))]
    if settings.node.waglaylad_daemon_storage_folder_enable {
        storage.track_storage_root(Some(settings.node.waglaylad_daemon_storage_folder.as_str()));
    }

    let mut this = Self {
      is_shutdown_pending: false,
      account_updated: false,
      settings_storage_requested: false,
      last_settings_storage_request: Instant::now(),
      manager: manager.clone(),
      wallet: manager.wallet().clone(),  // Assuming runtime has a wallet() method
      application_events_channel: manager.application_events().clone(),  // Assuming this method exists
      stack: VecDeque::new(),
      prev_components: None,
      component: components.get(&TypeId::of::<CreateWallet>()).unwrap().clone(),
      components: components.clone(),
      settings,
      mobile_style,
      default_style,
      wallet_descriptor: None,
      wallet_list: Vec::new(),
      prv_key_data_map: None,
      user_accounts: None,
      current_account: None,
      node_state: Default::default(),
      hint: None,
      discard_hint: false,
      exception: None,
      // release: None,
      window_frame,
      storage,
      node_info: None,
      footer,
      daemon_console: components.get(&TypeId::of::<DaemonConsole>()).unwrap().clone(),
    };

    components.values().for_each(|component| {
      component.init(&mut this);
    });

    this.wallet_update_list();

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
        //     this.register_visibility_handler();
        } else {
            let storage = this.storage.clone();
            tokio::spawn(async move {
                loop {
                    storage.update();
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            });
        }
    }

    this
  }

  pub fn get_mut<T>(&mut self) -> RefMut<'_, T>
  where
    T: ComponentT + 'static,
  {
    let cell = self.components.get_mut(&TypeId::of::<T>()).unwrap();
    RefMut::map(cell.inner.module.borrow_mut(), |r| {
      (r).as_any_mut()
        .downcast_mut::<T>()
        .expect("unable to downcast_mut module")
    })
  }
}

impl eframe::App for Core {
  fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
      egui::Rgba::TRANSPARENT.to_array() // Keep transparent background
  }

  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    for event in self.application_events_channel.iter() {
      if let Err(err) = self.handle_events(event.clone(), ctx, frame) {
        log_error!("error processing wallet runtime event: {}", err);
      }
    }

    if self.account_updated {
      let current_account = self.current_account.clone();
      let wallet_descriptor = self.wallet_descriptor.clone();

      self.account_updated = false;

      let mut view_wallet = self.get_mut::<ViewWallet>();
      view_wallet.update_biscuit_account(
        ctx, 
        &current_account, 
        &wallet_descriptor
      );
    }

    if self.is_shutdown_pending {
      return;
    }

    if self.settings_storage_requested
      && self.last_settings_storage_request.elapsed() > Duration::from_secs(5)
    {
      self.settings_storage_requested = false;
      self.settings.store_sync().unwrap();
      println!("saving settings");
    }

    self.render_frame(ctx, frame);

    ctx.request_repaint_after(std::time::Duration::from_secs_f32(1.0 / 60.0));
  }

  // Optionally, include a basic exit handler
  #[cfg(not(target_arch = "wasm32"))]
  fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
    println!("Goodbye!");
  }
}

impl Core {
  pub fn render_frame(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    window_frame(self.window_frame, ctx, "Waglayla Wag-DX", |ui| {
      if !self.settings.initialized {
        egui::CentralPanel::default()
        .frame(create_custom_popup(ctx))
        .show_inside(ui, |ui| {
          self.components
            .get(&TypeId::of::<Welcome>())
            .unwrap()
            .clone()
            .render(self, ctx, frame, ui);
        });

        return;
      }

      let available_rect = ui.available_rect_before_wrap();
      let footer_height = 28.0;
            
      let main_rect = egui::Rect::from_min_max(
        available_rect.min,
        egui::pos2(available_rect.right(), available_rect.bottom() - footer_height)
      );
    
      let footer_rect = egui::Rect::from_min_max(
        egui::pos2(available_rect.left(), available_rect.bottom() - footer_height),
        available_rect.max,
      );

      ui.allocate_ui_at_rect(footer_rect, |ui| {
        let mut components_clone = self.components.clone();
        let footer_component = components_clone.get_mut(&TypeId::of::<Footer>()).unwrap();
        footer_component.render(self, ctx, frame, ui);
      });

      ui.allocate_ui_at_rect(main_rect, |ui| {
          // Render sidebar
          let outline_type_id = TypeId::of::<Outline>();
          let outline = self.components.get(&outline_type_id).cloned();
          if let Some(outline) = self.components.get(&TypeId::of::<Outline>()) {
            let mut components_clone = self.components.clone();
            let outline_component = components_clone.get_mut(&outline_type_id).unwrap();
            outline_component.render(self, ctx, frame, ui);
          }

          // Render active component with a persistent state
          let active_component = self.component.clone();
          let content_rect = ui.available_rect_before_wrap();
          ui.allocate_ui_at_rect(content_rect, |ui| {
              // We clone components to avoid the borrow checker issue
              let mut components_clone = self.components.clone();
              let component_type_id = active_component.type_id();
              let active_component_mut = components_clone.get_mut(&component_type_id).unwrap();
              
              active_component_mut.render(self, ctx, frame, ui);
          });
      });
    });
  }

  pub fn set_active_component<T: ComponentT + 'static>(&mut self) {
    if let Some(component) = self.components.get(&TypeId::of::<T>()) {
      self.component = component.clone();
    }
  }

  pub fn set_active_component_by_type(&mut self, type_id: TypeId) {
    if let Some(component) = self.components.get(&type_id).cloned() {
      self.component = component;
    }
  }
  // tokio::spawn(async move {
  //   let result: Result<()> = async {
  //     wallet.accounts_activate(Some(account_ids)).await?;
  //     Ok(())
  //   }
  //   .await;
  // });
  pub fn select_account(&mut self, account: Option<Account>, notify : bool) {
    if let Some(account) = account {
      if notify {
        self.current_account = Some(account.clone());
        let account_id = account.id();
        tokio::spawn(async move {
          let result: Result<()> = async {
            manager().wallet().accounts_select(Some(account_id)).await?;
            Ok(())
          }
          .await;

          result
        });
        self.account_updated = true;
      }
    } else {
      // TODO
      // self.state = AccountManagerState::Select;
      // self.context.loading = false;

      // if notify {
      //   spawn(async move {
      //     self.wallet.accounts_select(None).await?;
      //     Ok(())
      //   });
      // }
    }
  }

  pub fn user_accounts(&self) -> &Option<AccountGroup> {
    &self.user_accounts
  }

  pub fn get_component<T: ComponentT + 'static>(&self) -> Option<&Component> {
    self.components.get(&TypeId::of::<T>())
  }

  pub fn get_component_mut<T: ComponentT + 'static>(&mut self) -> Option<&mut Component> {
    self.components.get_mut(&TypeId::of::<T>())
  }
  
  pub fn sender(&self) -> crate::dx_manager::channel::Sender<Events> {
    self.application_events_channel.sender.clone()
  }

  pub fn wallet(&self) -> Arc<dyn WalletApi> {
    self.wallet.clone()
  }

  pub fn node_state(&self) -> NodeState {
    self.node_state.clone()
  }

  pub fn purge_secure_stack(&mut self) {
    self.stack.retain(|module| !module.secure());
  }

  pub fn prv_key_data_map(&self) -> &Option<HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>> {
    &self.prv_key_data_map
  }

  pub fn store_settings(&self) {
    self.application_events_channel
      .sender
      .try_send(Events::StoreSettings)
      .unwrap();
  }

  fn update_wallet(&self) {
    // if let Some(user_accounts) = self.user_accounts.as_ref() {
    //   if let Some(updated_account) = user_accounts.get(&account.id()) {
    //     self.current_account = Some(updated_account.clone());
    //   } else {
    //     self.set_active_component_by_type(crate::components::CreateWallet);
    //   }
    // }
  }

  pub fn handle_events(
      &mut self,
      event: Events,
      ctx: &egui::Context,
      _frame: &mut eframe::Frame,
  ) -> Result<()> {
    match event {
        Events::Exit => {
            cfg_if! {
                if #[cfg(not(target_arch = "wasm32"))] {
                    self.is_shutdown_pending = true;
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }

        Events::WalletList { wallet_list } => {
          self.wallet_list.clone_from(&*wallet_list);
          self.wallet_list.sort();
        }

        Events::PeerCountUpdate(count) => {
          self.node_state.node_peers = Some(count);
        }

        Events::WalletUpdate => {
          self.update_wallet();
        }

        Events::PrvKeyDataInfo {
          prv_key_data_info_map,
        } => {
          self.prv_key_data_map = Some(prv_key_data_info_map);
        }

        Events::StoreSettings => {
          self.settings_storage_requested = true;
          self.last_settings_storage_request = Instant::now();
        }

        Events::Wallet { event } => {
          // println!("wallet event: {:?}", event);
          match *event {
            CoreWallet::WalletPing => {
                // log_info!("received wallet ping event...");
                // crate::runtime::runtime().notify(UserNotification::info("Wallet ping"));
            }
            CoreWallet::Metrics {
              network_id: _,
              metrics,
            } => {
              println!("Wagg-DX - received metrics event {metrics:?}");

              match metrics {
                MetricsUpdate::WalletMetrics {
                  mempool_size,
                  node_peers: peers,
                  network_tps: tps,
                } => {
                  self.sender().try_send(Events::MempoolSize {
                    mempool_size: mempool_size as usize,
                  })?;

                  self.node_state.node_peers = Some(peers as usize);
                  self.node_state.node_mempool_size = Some(mempool_size as usize);
                  self.node_state.network_tps = Some(tps);
                }
              }
            }
            CoreWallet::Error { message } => {
              // runtime().notify(UserNotification::error(message.as_str()));
              println!("{message}");
            }
            CoreWallet::UtxoProcStart => {
              self.node_state.error = None;

              if self.node_state().is_open() {
                let wallet = self.wallet().clone();
                tokio::spawn(async move {
                  let result: Result<()> = async {
                    wallet.wallet_reload(false).await?;
                    Ok(())
                  }
                  .await;

                  result
                });
              }
            }
            CoreWallet::UtxoProcStop => {}
            CoreWallet::UtxoProcError { message } => {
              // runtime().notify(UserNotification::error(message.as_str()));

              // if message.contains("network type") {
              //   self.node_state.error = Some(message);
              // }
            }
            #[allow(unused_variables)]
            CoreWallet::Connect { url, network_id } => {
              // log_info!("Connected to {url:?} on network {network_id}");
              self.node_state.is_connected = true;
              self.node_state.url = url;
              self.node_state.network_id = Some(network_id);
            }
            #[allow(unused_variables)]
            CoreWallet::Disconnect {
              url: _,
              network_id: _,
            } => {
              self.node_state.is_connected = false;
              self.node_state.sync_state = None;
              self.node_state.is_synced = None;
              self.node_state.server_version = None;
              self.node_state.url = None;
              self.node_state.network_id = None;
              self.node_state.current_daa_score = None;
              self.node_state.error = None;
              self.node_state.node_metrics = None;
              self.node_state.node_peers = None;
              self.node_state.node_mempool_size = None;
              // self.network_pressure.clear();
            }
            CoreWallet::UtxoIndexNotEnabled { url } => {
              // self.exception = Some(Exception::UtxoIndexNotEnabled { url });
            }
            CoreWallet::SyncState { sync_state } => {
              println!("Sync State: {:?}", sync_state);
              self.node_state.sync_state = Some(sync_state);
            }
            CoreWallet::ServerStatus {
              is_synced,
              server_version,
              url,
              network_id,
            } => {
              self.node_state.is_synced = Some(is_synced);
              self.node_state.server_version = Some(server_version);
              self.node_state.url = url;
              self.node_state.network_id = Some(network_id);
            }
            CoreWallet::WalletHint { hint } => {
              self.hint = hint;
              self.discard_hint = false;
            }
            CoreWallet::WalletReload {
              wallet_descriptor,
              account_descriptors,
            } => {
              self.node_state.is_open = true;

              self.wallet_descriptor = wallet_descriptor;
              let network_id = self
                .node_state
                .network_id
                .unwrap_or(Network::Mainnet.into());
              let account_descriptors =
                account_descriptors.ok_or(Error::WalletOpenAccountDescriptors)?;
              self.load_accounts(network_id, account_descriptors)?;
            }
            CoreWallet::WalletOpen {
                wallet_descriptor,
                account_descriptors,
            } => {
              self.node_state.is_open = true;

              self.wallet_descriptor = wallet_descriptor;
              let network_id = self
                .node_state
                .network_id
                .unwrap_or(Network::Mainnet.into());
              let account_descriptors =
                account_descriptors.ok_or(Error::WalletOpenAccountDescriptors)?;
              self.load_accounts(network_id, account_descriptors)?;

              if let Some(first) = self.user_accounts.clone().expect("load_accounts done").first() {
                self.current_account = Some(first.clone());
                self.account_updated = true;
              }
            }
            CoreWallet::WalletCreate {
              wallet_descriptor,
              storage_descriptor: _,
            } => {
              self.wallet_list.push(wallet_descriptor.clone());
              self.wallet_descriptor = Some(wallet_descriptor);
              self.user_accounts = Some(AccountGroup::default());
              self.node_state.is_open = true;
            }
            CoreWallet::PrvKeyDataCreate { prv_key_data_info } => {
              if let Some(prv_key_data_map) = self.prv_key_data_map.as_mut() {
                prv_key_data_map
                  .insert(*prv_key_data_info.id(), Arc::new(prv_key_data_info));
              } else {
                let mut prv_key_data_map = HashMap::new();
                prv_key_data_map
                  .insert(*prv_key_data_info.id(), Arc::new(prv_key_data_info));
                self.prv_key_data_map = Some(prv_key_data_map);
              }
            }
            CoreWallet::AccountDeactivation { ids: _ } => {}
            CoreWallet::AccountActivation { ids: _ } => {}
            CoreWallet::AccountCreate {
              account_descriptor: _,
            } => {}
            CoreWallet::AccountUpdate { account_descriptor } => {
              let account_id = account_descriptor.account_id();
              if let Some(user_accounts) = self.user_accounts.as_ref() {
                if let Some(account) = user_accounts.get(account_id) {
                  account.update(account_descriptor);
                }
              }
            }
            CoreWallet::WalletError { message: _ } => {}
            CoreWallet::WalletClose => {
              self.hint = None;
              self.node_state.is_open = false;
              self.user_accounts = None;
              self.wallet_descriptor = None;
              self.prv_key_data_map = None;

              self.components.clone().into_iter().for_each(|(_, module)| {
                module.reset(self);
              });

              self.purge_secure_stack();
            }
            CoreWallet::AccountSelection { id } => {
              if let Some(user_accounts) = self.user_accounts.as_ref() {
                if let Some(id) = id {
                  if let Some(account) = user_accounts.get(&id) {
                    let account = account.clone();
                    // let device = self.device().clone();
                    let wallet = self.wallet();
                    // log_info!("--- selecting account: {id:?}");
                    self.current_account = Some(account);
                    // self.get_mut::<modules::AccountManager>().select(
                    //   wallet,
                    //   Some(account),
                    //   device,
                    //   false,
                    // );
                  }
                }
              }
            }
            CoreWallet::DaaScoreChange { current_daa_score } => {
              self.node_state.current_daa_score.replace(current_daa_score);
            }
            // Ignore scan notifications
            CoreWallet::Discovery { record } => match record.binding().clone() {
              Binding::Account(id) => {
                // self.user_accounts
                //   .as_ref()
                //   .and_then(|user_accounts| {
                //     user_accounts.get(&id).map(|account| {
                //       if account
                //         .transactions()
                //         .replace_or_insert(Transaction::new_confirmed(
                //           Arc::new(record),
                //         ))
                //         .is_none()
                //       {
                //         let mut binding = account.transactions();
                //         while binding.len() as u64 > TRANSACTION_PAGE_SIZE {
                //           binding.pop();
                //         }
                //         account.set_transaction_count(
                //           account.transaction_count() + 1,
                //         );
                //       }
                //     })
                //   });
              }
              Binding::Custom(_) => {
                log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
              }
            }
            // Ignore stasis notifications
            CoreWallet::Stasis { record: _ } => {}
            // A transaction has been confirmed
            CoreWallet::Maturity { record } => {
              if record.is_change() {
                return Ok(());
              }

              match record.binding().clone() {
                Binding::Account(id) => {
                  // self.user_accounts
                  //   .as_ref()
                  //   .and_then(|user_accounts| {
                  //     user_accounts.get(&id).map(|account| {
                  //       if account
                  //         .transactions()
                  //         .replace_or_insert(Transaction::new_confirmed(
                  //           Arc::new(record),
                  //         ))
                  //         .is_none()
                  //       {
                  //         let mut binding = account.transactions();
                  //         while binding.len() as u64 > TRANSACTION_PAGE_SIZE {
                  //           binding.pop();
                  //         }
                  //         account.set_transaction_count(
                  //           account.transaction_count() + 1,
                  //         );
                  //       }
                  //     })
                  //   });
                }
                Binding::Custom(_) => {
                  log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
                }
              }
            }
            
            CoreWallet::Pending { record } => match record.binding().clone() {
                Binding::Account(id) => {
                  // self.user_accounts
                  //   .as_ref()
                  //   .and_then(|user_accounts| {
                  //     user_accounts.get(&id).map(|account| {
                  //       if account
                  //         .transactions()
                  //         .replace_or_insert(Transaction::new_processing(
                  //           Arc::new(record),
                  //         ))
                  //         .is_none()
                  //       {
                  //         let mut binding = account.transactions();
                  //         while binding.len() as u64 > TRANSACTION_PAGE_SIZE {
                  //           binding.pop();
                  //         }
                  //         account.set_transaction_count(
                  //           account.transaction_count() + 1,
                  //         );
                  //       }
                  //     })
                  //   });
                }
                Binding::Custom(_) => {
                  log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
                }
            }

            CoreWallet::Reorg { record } => match record.binding().clone() {
              Binding::Account(id) => {
                // self.user_accounts
                //   .as_mut()
                //   .and_then(|user_accounts| {
                //     user_accounts
                //       .get(&id)
                //       .map(|account| account.transactions().remove(record.id()))
                //   });
              }
              Binding::Custom(_) => {
                log_error!("Error while processing transaction {}: custom bindings are not supported", record.id());
              }
            }

            CoreWallet::Balance { balance, id } => {
              if let Some(user_accounts) = &self.user_accounts {
                if let Some(account) = user_accounts.get(&id.into()) {
                  account.update_balance(balance)?;
                } else {
                  log_error!("unable to find account {}", id);
                }
              } else {
                log_error!(
                  "received CoreWallet::Balance while account collection is empty"
                );
              }
            }
          }
        }
        // Events::Error(error) => {
        //     manager().notify(UserNotification::error(error.as_str()));
        // }
        _ => {}
    }

    Ok(())
  }

  pub fn handle_account_creation(
    &mut self,
    account_descriptors: Vec<AccountDescriptor>,
  ) -> Vec<Account> {
    let accounts = account_descriptors
      .into_iter()
      .map(|account_descriptor| Account::from(account_descriptor))
      .collect::<Vec<_>>();

    self.user_accounts
      .as_mut()
      .expect("account collection")
      .extend_unchecked(accounts.clone());

    let account_ids = accounts
      .iter()
      .map(|account| account.id())
      .collect::<Vec<_>>();

    let wallet = self.wallet().clone();
    tokio::spawn(async move {
      let result: Result<()> = async {
        wallet.accounts_activate(Some(account_ids)).await?;
        Ok(())
      }
      .await;
    });

    if let Some(first) = accounts.first() {
      // let device = self.device().clone();
      // let wallet = self.wallet();
      self.current_account = Some(first.clone());
      // self.get_mut::<modules::AccountManager>().select(
      //   wallet,
      //   Some(first.clone()),
      //   device,
      //   true,
      // );
    }

    accounts
  }

  pub fn wallet_update_list(&self) {
    let manager = self.manager.clone();
    tokio::spawn(async move {
      let wallet_list = manager.wallet().wallet_enumerate().await?;
      manager
        .send(Events::WalletList {
          wallet_list: Arc::new(wallet_list),
        })
        .await?;
      Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    });
  }

  fn load_accounts(
      &mut self,
      network_id: NetworkId,
      account_descriptors: Vec<AccountDescriptor>,
  ) -> Result<()> {
      let application_events_sender = self.application_events_channel.sender.clone();

      let account_list = account_descriptors
        .into_iter()
        .map(|account_descriptor| Account::from(account_descriptor))
        .collect::<Vec<_>>();

      self.user_accounts = Some(account_list.clone().into());

      let manager = self.manager.clone();
      tokio::spawn(async move {
        let result: Result<()> = async {
          let prv_key_data_info_map = manager
            .wallet()
            .prv_key_data_enumerate()
            .await?
            .clone()
            .into_iter()
            .map(|prv_key_data_info| (*prv_key_data_info.id(), prv_key_data_info))
            .collect::<HashMap<_, _>>();

          application_events_sender
            .send(Events::PrvKeyDataInfo {
              prv_key_data_info_map,
            })
            .await?;

          let account_ids = account_list
            .iter()
            .map(|account| account.id())
            .collect::<Vec<_>>();

          let account_map: HashMap<AccountId, Account> = account_list
            .clone()
            .into_iter()
            .map(|account| (account.id(), account))
            .collect::<HashMap<_, _>>();

          // TODO - finish progressive transaction loading implementation
          // let futures = account_ids
          //   .into_iter()
          //   .map(|account_id| {
          //     runtime.wallet().transactions_data_get_range(
          //       account_id,
          //       network_id,
          //       0..TRANSACTION_PAGE_SIZE,
          //     )
          //   })
          //   .collect::<Vec<_>>();

          // let transaction_data = join_all(futures)
          //   .await
          //   .into_iter()
          //   .map(|v| v.map_err(Error::from))
          //   .collect::<Result<Vec<_>>>()?;

          // transaction_data.into_iter().for_each(|data| {
          //     let TransactionsDataGetResponse {
          //         account_id,
          //         transactions,
          //         start: _,
          //         total,
          //     } = data;

          //     if let Some(account) = account_map.get(&account_id) {
          //         if let Err(err) = account.load_transactions(transactions, total) {
          //             log_error!("error loading transactions into account {account_id}: {err}");
          //         }
          //     } else {
          //         log_error!("unable to find account {}", account_id);
          //     }
          // });

          manager.wallet().accounts_activate(None).await?;
          application_events_sender.send(Events::WalletUpdate).await?;

          Ok(())
        }
        .await;
      });

      Ok(())
  }
}