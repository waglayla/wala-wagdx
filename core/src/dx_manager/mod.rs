use crate::imports::*;
// use system::*;

pub struct Inner {
  egui_ctx: egui::Context,
  is_running: Arc<AtomicBool>,
  start_time: Instant,
  // system: Option<System>,

  // waglayla: Arc<WaglaylaService>,
}

#[derive(Clone)]
pub struct DXManager {
  inner: Arc<Inner>,
}

impl DXManager {
  pub fn new(ctx: egui::Context) -> Self {
    let runtime = Self {
      inner: Arc::new(Inner {
        egui_ctx: ctx.clone(),
        is_running: Arc::new(AtomicBool::new(false)),
        start_time: Instant::now(),
        // system: Some(system),
      }),
    };
  
    assign_manager(Some(runtime.clone()));
    runtime
  }

  pub fn request_repaint(&self) {
    self.inner.egui_ctx.request_repaint();
  }

  pub fn drop(&self) {
    assign_manager(None);
  }
}

static MANAGER_MUTEX: Mutex<Option<DXManager>> = Mutex::new(None);

fn assign_manager(runtime: Option<DXManager>) {
  match runtime {
    Some(runtime) => {
      let mut global = MANAGER_MUTEX.lock().unwrap();
      if global.is_some() {
        panic!("runtime already initialized");
      }
      global.replace(runtime);
    }
    None => {
      MANAGER_MUTEX.lock().unwrap().take();
    }
  };
}
