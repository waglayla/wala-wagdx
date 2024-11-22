use crate::frame::window_frame;
use crate::imports::*;
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

pub enum Exception {
  #[allow(dead_code)]
  UtxoIndexNotEnabled { url: Option<String> },
}

pub struct Core {
  is_shutdown_pending: bool,
  settings_storage_requested: bool,
  last_settings_storage_request: Instant,

  // runtime: Runtime,
  // wallet: Arc<dyn WalletApi>,
  // application_events_channel: ApplicationEventsChannel,
  // deactivation: Option<Module>,
  // component: Module,
  // components: HashMap<TypeId, Module>,
  // pub stack: VecDeque<Module>,
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
  // pub account_collection: Option<AccountCollection>,
  // pub release: Option<Release>,

  // pub device: Device,
  // pub market: Option<Market>,
  // pub debug: bool,
  pub window_frame: bool,
  // callback_map: CallbackMap,
  // pub network_pressure: NetworkPressure,
  // notifications: Notifications,
  // pub storage: Storage,
  // pub feerate : Option<Arc<RpcFeeEstimate>>,
  // pub feerate: Option<FeerateEstimate>, TODO maybe
  pub node_info: Option<Box<String>>,
}

impl Core {
  pub fn new(
    cc: &eframe::CreationContext<'_>,
    settings: Settings,
    window_frame: bool,
  ) -> Self {
    // Initialize fonts if needed
    crate::fonts::init_fonts(cc);
    
    // Create default styles
    let mut default_style = (*cc.egui_ctx.style()).clone();
    let mut mobile_style = (*cc.egui_ctx.style()).clone();
    
    // Apply your theme
    apply_theme_by_name(
      &cc.egui_ctx,
      settings.user_interface.theme_color.as_str(),
      settings.user_interface.theme_style.as_str(),
    );

    Self {
      is_shutdown_pending: false,
      settings_storage_requested: false,
      last_settings_storage_request: Instant::now(),
      // runtime,
      // wallet: runtime.wallet().clone(),  // Assuming runtime has a wallet() method
      // application_events_channel: runtime.application_events().clone(),  // Assuming this method exists
      // stack: VecDeque::new(),
      settings,
      mobile_style,
      default_style,
      wallet_descriptor: None,
      wallet_list: Vec::new(),
      prv_key_data_map: None,
      // account_collection: None,
      // release: None,
      window_frame,
      node_info: None,
    }
  }
}

impl eframe::App for Core {
  fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
      egui::Rgba::TRANSPARENT.to_array() // Keep transparent background
  }

  fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
    // Update device screen size if needed
    // self.device_mut().set_screen_size(&ctx.screen_rect());
    
    // Call your render_frame method
    self.render_frame(ctx, frame);
  }

  // Optionally, include a basic exit handler
  #[cfg(not(target_arch = "wasm32"))]
  fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
    println!("Goodbye!");
  }
}

impl Core {
  fn render_frame(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    window_frame(self.window_frame, ctx, "Waglayla Wag-DX", |ui| {
      ui.label("Hello, World!");
    });
  }
}