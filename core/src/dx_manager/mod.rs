cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
      pub mod signal_handler;
      // pub mod panic;
  } else {
      // ...
  }
}

pub mod channel;
pub mod services;

use crate::imports::*;
pub use services::Service;
use services::*;
// use system::*;

pub struct Inner {
  services: Mutex<Vec<Arc<dyn Service>>>,
  application_events: ApplicationEventsChannel,
  egui_ctx: egui::Context,
  is_running: Arc<AtomicBool>,
  start_time: Instant,
  // system: Option<System>,

  waglayla: Arc<WaglaylaService>,
  daemon_channel : Channel<DaemonMessage>,
}

#[derive(Clone)]
pub struct DXManager {
  inner: Arc<Inner>,
}

impl DXManager {
  pub fn new(
    ctx: &egui::Context,
    application_events: Option<ApplicationEventsChannel>,
    wallet_api: Option<Arc<dyn WalletApi>>,
    settings: &Settings,
    daemon_channel: Channel<DaemonMessage>
  ) -> Self {
    let application_events =
      application_events.unwrap_or_else(ApplicationEventsChannel::unbounded);

    let waglayla = Arc::new(WaglaylaService::new(
      application_events.clone(),
      &settings, 
      daemon_channel.sender.clone(),
      wallet_api
    ));
    let services: Mutex<Vec<Arc<dyn Service>>> = Mutex::new(vec![
        waglayla.clone(),
    ]);

    let manager = Self {
      inner: Arc::new(Inner {
        services,
        application_events: application_events,
        egui_ctx: ctx.clone(),
        is_running: Arc::new(AtomicBool::new(false)),
        start_time: Instant::now(),
        waglayla,
        daemon_channel: daemon_channel.clone(),
        // system: Some(system),
      }),
    };

    assign_manager(Some(manager.clone()));
    manager
  }

  pub fn request_repaint(&self) {
    self.inner.egui_ctx.request_repaint();
  }

  pub fn drop(&self) {
    assign_manager(None);
  }

  pub fn error(&self, text: impl Into<String>) {
      // self.inner
      //     .application_events
      //     .sender
      //     .try_send(Events::Notify {
      //         user_notification: UserNotification::error(text),
      //     })
      //     .ok();
  }

  pub fn start_services(&self) {
    let services = self.services();
    for service in services {
      tokio::spawn(async move {
        if let Err(e) = service.clone().launch().await {
          eprintln!("Service {} failed: {}", service.name(), e);
        }
      });
    }
  }

  pub fn start(&self) {
    self.inner.is_running.store(true, Ordering::SeqCst);
    self.start_services();
  }

  pub fn stop_services(&self) {
    self.services()
      .into_iter()
      .for_each(|service| service.terminate());
  }

  pub async fn join_services(&self) {
    let futures = self
      .services()
      .into_iter()
      .map(|service| service.join())
      .collect::<Vec<_>>();
    futures::future::join_all(futures).await;
  }

  pub async fn shutdown(&self) {
    if self.inner.is_running.load(Ordering::SeqCst) {
      self.inner.is_running.store(false, Ordering::SeqCst);
      // self.inner.daemon_channel.sender.close();
      // while let Ok(_) = self.inner.daemon_channel.receiver.try_recv() {}
      self.stop_services();
      self.join_services().await;
      assign_manager(None);
    }
  }

  // GETTERS
  pub fn uptime(&self) -> Duration {
    self.inner.start_time.elapsed()
  }

  pub fn services(&self) -> Vec<Arc<dyn Service>> {
    self.inner.services.lock().unwrap().clone()
  }

  pub fn waglayla_service(&self) -> &Arc<WaglaylaService> {
    &self.inner.waglayla
  }

  pub fn wallet(&self) -> Arc<dyn WalletApi> {
    self.inner.waglayla.wallet()
  }

  pub fn application_events(&self) -> &ApplicationEventsChannel {
    &self.inner.application_events
  }

  // SETTER/SENDERS
  pub async fn send(&self, msg: Events) -> Result<()> {
    self.inner.application_events.sender.send(msg).await?;
    Ok(())
  }

  pub fn try_send(&self, msg: Events) -> Result<()> {
    self.inner.application_events.sender.try_send(msg)?;
    Ok(())
  }

  /// Update storage size
  pub fn update_storage(&self, options: StorageUpdateOptions) {
    self.inner
      .application_events
      .sender
      .try_send(Events::UpdateStorage(options))
      .ok();
  }
}

static MANAGER_MUTEX: Mutex<Option<DXManager>> = Mutex::new(None);

fn assign_manager(runtime: Option<DXManager>) {
  match runtime {
    Some(runtime) => {
      let mut global = MANAGER_MUTEX.lock().unwrap();
      if global.is_some() {
        panic!("DXManager already initialized");
      }
      global.replace(runtime);
    }
    None => {
      MANAGER_MUTEX.lock().unwrap().take();
    }
  };
}

pub fn try_manager() -> Option<DXManager> {
  MANAGER_MUTEX.lock().unwrap().clone()
}

pub fn manager() -> DXManager {
  MANAGER_MUTEX
    .lock()
    .unwrap()
    .clone()
    .expect("DXManager not initialized")
}

#[cfg(not(target_arch = "wasm32"))]
pub fn halt() {
    if let Some(runtime) = try_manager() {
        runtime.try_send(Events::Exit).ok();
        runtime.waglayla_service().clone().terminate();

        let handle = tokio::spawn(async move { runtime.shutdown().await });

        while !handle.is_finished() {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }
}


#[cfg(not(target_arch = "wasm32"))]
pub fn abort() {
    const TIMEOUT: u128 = 5000;
    let flag = Arc::new(AtomicBool::new(false));
    let flag_ = flag.clone();
    let thread = std::thread::Builder::new()
        .name("halt".to_string())
        .spawn(move || {
            let start = std::time::Instant::now();
            while !flag_.load(Ordering::SeqCst) {
                if start.elapsed().as_millis() > TIMEOUT {
                    println!("halting...");
                    std::process::exit(1);
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        })
        .ok();

    halt();

    flag.store(true, Ordering::SeqCst);
    if let Some(thread) = thread {
        thread.join().unwrap();
    }

    #[cfg(feature = "console")]
    {
        println!("Press Enter to exit...");
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
    }

    std::process::exit(1);
}
