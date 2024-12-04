use std::any::type_name;

use crate::imports::*;

pub mod outline;
pub mod hello;
pub mod blank;
pub mod console;
pub mod welcome;
pub mod settings;
pub mod footer;
pub mod wallet_ui;

// waglayla_ng_macros::register_modules!(
//     register_generic_modules,
//     [
//         account_create,
//         account_manager,
//         donations,
//         export,
//         import,
//         overview,
//         private_key_create,
//         request,
//         scanner,
//         settings,
//         testing,
//         wallet_create,
//         wallet_open,
//         wallet_secret,
//         welcome,
//     ]
// );

// #[cfg(not(target_arch = "wasm32"))]
// waglayla_ng_macros::register_modules!(register_native_modules, [changelog, logs, node,]);

// #[cfg(not(feature = "lean"))]
// waglayla_ng_macros::register_modules!(register_advanced_modules, [block_dag, metrics,]);

pub enum ComponentStyle {
  Mobile,
  Default,
}

pub enum ComponentCaps {
  Desktop,
  Mobile,
  WebApp,
  Extension,
}

pub trait WizardActionTrait {
  fn is_no_action(&self) -> bool;
  fn is_back(&self) -> bool;
  fn from_back() -> Self;
}

pub trait ComponentT: Downcast {
  fn name(&self) -> Option<&'static str> {
    None
  }

  fn modal(&self) -> bool {
    false
  }

  fn secure(&self) -> bool {
    false
  }

  fn style(&self) -> ComponentStyle {
    // ComponentStyle::Large
    ComponentStyle::Default
  }

  fn status_bar(&self, _core: &mut Core, _ui: &mut Ui) {}
  fn activate(&mut self, _core: &mut Core) {}
  fn deactivate(&mut self, _core: &mut Core) {}
  fn reset(&mut self, _core: &mut Core) {}
  // fn connect(&mut self, _core: &mut Core, _network: Network) {}
  // fn disconnect(&mut self, _core: &mut Core) {}
  // fn network_change(&mut self, _core: &mut Core, _network: Network) {}
  fn hide(&mut self, _core: &mut Core) {}
  fn show(&mut self, _core: &mut Core) {}

  fn init(&mut self, _core: &mut Core) {}

  fn render(
    &mut self,
    core: &mut Core,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  );

  fn shutdown(&mut self) {}
}

impl_downcast!(ComponentT);

pub struct Inner {
  pub name: String,
  pub type_name: String,
  pub type_id: TypeId,
  pub module: Rc<RefCell<dyn ComponentT>>,
}

#[derive(Clone)]
pub struct Component {
  pub inner: Rc<Inner>,
}

impl Component {
  pub fn init(&self, core: &mut Core) {
    self.inner.module.borrow_mut().init(core)
  }

  pub fn activate(&self, core: &mut Core) {
    self.inner.module.borrow_mut().activate(core)
  }

  pub fn deactivate(&self, core: &mut Core) {
    self.inner.module.borrow_mut().deactivate(core)
  }

  pub fn reset(&self, core: &mut Core) {
    self.inner.module.borrow_mut().reset(core)
  }

  // pub fn connect(&self, core: &mut Core, network: Network) {
  //   self.inner.module.borrow_mut().connect(core, network)
  // }

  // pub fn disconnect(&self, core: &mut Core) {
  //   self.inner.module.borrow_mut().disconnect(core)
  // }

  // pub fn network_change(&self, core: &mut Core, network: Network) {
  //   self.inner.module.borrow_mut().network_change(core, network)
  // }

  pub fn hide(&self, core: &mut Core) {
    self.inner.module.borrow_mut().hide(core)
  }

  pub fn show(&self, core: &mut Core) {
    self.inner.module.borrow_mut().show(core)
  }

  pub fn status_bar(&self, core: &mut Core, ui: &mut Ui) {
    self.inner.module.borrow_mut().status_bar(core, ui)
  }

  pub fn render(
    &self,
    core: &mut Core,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    let mut module = self.inner.module.borrow_mut();

    match module.style() {
      ComponentStyle::Mobile => {
        ui.style_mut().text_styles = core.mobile_style.text_styles.clone();
      }
      ComponentStyle::Default => {
        ui.style_mut().text_styles = core.default_style.text_styles.clone();
      }
    }

    module.render(core, ctx, frame, ui)
  }

  pub fn render_default(
    &self,
    core: &mut Core,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    let mut module = self.inner.module.borrow_mut();

    module.render(core, ctx, frame, ui)
  }

  pub fn name(&self) -> &str {
    self.inner
      .module
      .borrow_mut()
      .name()
      .unwrap_or_else(|| self.inner.name.as_str())
  }

  pub fn modal(&self) -> bool {
    self.inner.module.borrow_mut().modal()
  }

  pub fn secure(&self) -> bool {
    self.inner.module.borrow_mut().secure()
  }

  pub fn type_id(&self) -> TypeId {
    self.inner.type_id
  }

  pub fn get<M>(&self) -> Ref<'_, M>
  where
    M: ComponentT + 'static,
  {
    Ref::map(self.inner.module.borrow(), |r| {
      (r).as_any()
        .downcast_ref::<M>()
        .expect("unable to downcast section")
    })
  }

  pub fn get_mut<M>(&mut self) -> RefMut<'_, M>
  where
    M: ComponentT + 'static,
  {
    RefMut::map(self.inner.module.borrow_mut(), |r| {
      (r).as_any_mut()
        .downcast_mut::<M>()
        .expect("unable to downcast_mut module")
    })
  }
}

impl std::fmt::Debug for Component {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.inner.name)
  }
}

impl Eq for Component {}

impl PartialEq for Component {
  fn eq(&self, other: &Self) -> bool {
    self.inner.type_id == other.inner.type_id
  }
}

impl<T> From<Rc<RefCell<T>>> for Component
where
  T: ComponentT + 'static,
{
  fn from(section: Rc<RefCell<T>>) -> Self {
    let type_name = type_name::<T>().to_string();
    let name = type_name.split("::").last().unwrap().to_string();
    let type_id = TypeId::of::<T>();
    Self {
      inner: Rc::new(Inner {
        name,
        type_name,
        type_id,
        module: section,
      }),
    }
  }
}

pub trait HashMapComponentExtension<T> {
  fn insert_typeid(&mut self, value: T)
  where
    T: ComponentT + 'static;
}

impl<T> HashMapComponentExtension<T> for HashMap<TypeId, Component>
where
  T: ComponentT,
{
  fn insert_typeid(&mut self, section: T) {
    let section = Rc::new(RefCell::new(section));
    self.insert(TypeId::of::<T>(), section.into());
  }
}
