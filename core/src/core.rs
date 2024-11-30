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

pub enum Exception {
  #[allow(dead_code)]
  UtxoIndexNotEnabled { url: Option<String> },
}

#[derive(Clone)]
pub struct Core {
  is_shutdown_pending: bool,
  settings_storage_requested: bool,
  last_settings_storage_request: Instant,
  manager: DXManager,
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

  // state: State,
  // hint: Option<Hint>,
  // discard_hint: bool,
  // exception: Option<Exception>,
  // screenshot: Option<Arc<ColorImage>>,

  pub wallet_descriptor: Option<WalletDescriptor>,
  pub wallet_list: Vec<WalletDescriptor>,
  pub prv_key_data_map: Option<HashMap<PrvKeyDataId, Arc<PrvKeyDataInfo>>>,
  // pub user_accounts: Option<UserWallet>,
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
    components.insert_typeid(Outline::default());
    components.insert_typeid(Hello::default());
    components.insert_typeid(Blank::default());
    components.insert_typeid(components::settings::Settings::new(manager.clone()));
    components.insert_typeid(components::wallet_ui::OpenWallet::new(manager.clone()));

    let daemon_console = DaemonConsole::new(daemon_receiver);
    components.insert_typeid(daemon_console);

    components.insert_typeid(Footer::default());
    let footer = components.get(&TypeId::of::<Footer>()).unwrap().clone();

    let hello_component = components.get(&TypeId::of::<Hello>()).unwrap().clone();

    let storage = Storage::default();
    #[cfg(not(target_arch = "wasm32"))]
    if settings.node.waglaylad_daemon_storage_folder_enable {
        storage.track_storage_root(Some(settings.node.waglaylad_daemon_storage_folder.as_str()));
    }

    let mut this = Self {
      is_shutdown_pending: false,
      settings_storage_requested: false,
      last_settings_storage_request: Instant::now(),
      manager: manager.clone(),
      wallet: manager.wallet().clone(),  // Assuming runtime has a wallet() method
      application_events_channel: manager.application_events().clone(),  // Assuming this method exists
      stack: VecDeque::new(),
      prev_components: None,
      component: hello_component,
      components: components.clone(),
      settings,
      mobile_style,
      default_style,
      wallet_descriptor: None,
      wallet_list: Vec::new(),
      prv_key_data_map: None,
      // user_accounts: None,
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

    self.render_frame(ctx, frame);
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
        egui::CentralPanel::default().show_inside(ui, |ui| {
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

      ui.allocate_ui_at_rect(footer_rect, |ui| {
          let mut components_clone = self.components.clone();
          let footer_component = components_clone.get_mut(&TypeId::of::<Footer>()).unwrap();
          footer_component.render(self, ctx, frame, ui);
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

  pub fn get_component<T: ComponentT + 'static>(&self) -> Option<&Component> {
      self.components.get(&TypeId::of::<T>())
  }

  pub fn get_component_mut<T: ComponentT + 'static>(&mut self) -> Option<&mut Component> {
      self.components.get_mut(&TypeId::of::<T>())
  }
  
  pub fn wallet(&self) -> Arc<dyn WalletApi> {
      self.wallet.clone()
  }

  pub fn handle_events(
      &mut self,
      event: Events,
      _ctx: &egui::Context,
      _frame: &mut eframe::Frame,
  ) -> Result<()> {
    match event {
        Events::Exit => {
            cfg_if! {
                if #[cfg(not(target_arch = "wasm32"))] {
                    self.is_shutdown_pending = true;
                    _ctx.send_viewport_cmd(ViewportCommand::Close);
                }
            }
        }

        Events::WalletList { wallet_list } => {
          self.wallet_list.clone_from(&*wallet_list);
          self.wallet_list.sort();
        }
        // Events::Error(error) => {
        //     manager().notify(UserNotification::error(error.as_str()));
        // }
        _ => {}
    }

    Ok(())
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
}