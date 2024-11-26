use crate::events::Events;
use crate::dx_manager::DXManager;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Signals {
  manager: DXManager,
  iterations: AtomicU64,
}

impl Signals {
  pub fn bind(manager: &DXManager) {
      let signals = Arc::new(Signals {
          manager: manager.clone(),
          iterations: AtomicU64::new(0),
      });

      ctrlc::set_handler(move || {
          let v = signals.iterations.fetch_add(1, Ordering::SeqCst);

          match v {
              0 => {
                  println!("^SIGTERM - shutting down...");
                  signals.manager.try_send(Events::Exit).unwrap_or_else(|e| {
                      println!("Error sending exit event: {:?}", e);
                  });
              }
              1 => {
                  println!("^SIGTERM - aborting...");
                  crate::dx_manager::abort();
              }
              _ => {
                  println!("^SIGTERM - halting");
                  std::process::exit(1);
              }
          }
      })
      .expect("Error setting signal handler");
  }
}
