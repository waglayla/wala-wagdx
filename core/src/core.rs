use crate::frame::window_frame;
use crate::imports::*;

use crate::components::HashMapComponentExtension;
use crate::components::outline::Outline;
use crate::components::hello::Hello;
use crate::components::blank::Blank;

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

pub struct Core {
  is_shutdown_pending: bool,
  settings_storage_requested: bool,
  last_settings_storage_request: Instant,
  // runtime: Runtime,
  // wallet: Arc<dyn WalletApi>,
  // application_events_channel: ApplicationEventsChannel,
  // deactivation: Option<Module>,
  component: Component,
  components: HashMap<TypeId, Component>,
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

    let mut components = HashMap::new();
    components.insert_typeid(Outline::default());
    components.insert_typeid(Hello::default());
    components.insert_typeid(Blank::default());

    let hello_component = components.get(&TypeId::of::<Hello>()).unwrap().clone();

    Self {
      is_shutdown_pending: false,
      settings_storage_requested: false,
      last_settings_storage_request: Instant::now(),
      // runtime,
      // wallet: runtime.wallet().clone(),  // Assuming runtime has a wallet() method
      // application_events_channel: runtime.application_events().clone(),  // Assuming this method exists
      // stack: VecDeque::new(),
      component: hello_component,
      components,
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
  pub fn render_frame(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
    window_frame(self.window_frame, ctx, "Waglayla Wag-DX", |ui| {
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

  // Method to get a component
  pub fn get_component<T: ComponentT + 'static>(&self) -> Option<&Component> {
      self.components.get(&TypeId::of::<T>())
  }

  // Method to get a mutable reference to a component
  pub fn get_component_mut<T: ComponentT + 'static>(&mut self) -> Option<&mut Component> {
      self.components.get_mut(&TypeId::of::<T>())
  }
}