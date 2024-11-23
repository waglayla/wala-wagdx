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
    let signals = Arc::new(Self {
      manager: manager.clone(),
      iterations: AtomicU64::new(0),
    });

    ctrlc::set_handler(move || match signals.iterations.fetch_add(1, Ordering::SeqCst) {
      0 => signals.manager.try_send(Events::Exit).unwrap_or_else(|e| println!("Exit error: {:?}", e)),
      1 => crate::dx_manager::abort(),
      _ => std::process::exit(1),
    }).expect("Signal handler error");
  }
}