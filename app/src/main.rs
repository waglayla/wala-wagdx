#![warn(clippy::all, rust_2018_idioms)]
// hide console window on Windows in release mode
#![cfg_attr(
    all(not(debug_assertions), not(feature = "console")),
    windows_subsystem = "windows"
)]

use cfg_if::cfg_if;
use wala_wagdx_core::app::{wagdx_main, ApplicationContext};
use wala_wagdx_core::gui::hello;

use workflow_log::*;

cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    fn run() {
      // Enable full backtrace in console mode
      #[cfg(feature = "console")] {
        std::env::set_var("RUST_BACKTRACE", "full");
      }

      // Initialize custom allocator
      waglayla_alloc::init_allocator_with_default_settings();

      // Define the async main body
      let body = async {
        if let Err(err) = wagdx_main(ApplicationContext::default()).await {
          log_error!("Error: {err}");
        }
      };

      // Run the async runtime
      #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
      tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(body);

      // Console mode: wait for user input before exiting
      #[cfg(feature = "console")]
      {
        println!("Press Enter to exit...");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
      }
    }
  } else {
    fn run() {
      wasm_bindgen_futures::spawn_local(async {
        if let Err(err) = wagdx_main(ApplicationContext::default()).await {
          log_error!("Error: {err}");
        }
      });
    }
  }
}

// cfg_if! {
//   if #[cfg(not(target_arch = "wasm32"))] {
//     pub fn run() {
//       #[cfg(feature = "console")] {
//         std::env::set_var("RUST_BACKTRACE", "full");
//       }

//       waglayla_alloc::init_allocator_with_default_settings();

//       let _ = hello::run_app();

//       // Console mode: wait for user input before exiting
//       #[cfg(feature = "console")]
//       {
//         println!("Press Enter to exit...");
//         let mut input = String::new();
//         std::io::stdin().read_line(&mut input).expect("Failed to read line");
//       }
//     }
//   } else {
//     #[cfg(target_arch = "wasm32")]
//     #[wasm_bindgen]
//     pub fn run() -> Result<(), eframe::wasm_bindgen::JsValue> {
//       let web_options = eframe::WebOptions::default();
//       eframe::start_web(
//         "canvas_id", // hardcode it
//         web_options,
//         Box::new(|_cc| Box::new(MyApp::default())),
//       )
//     }
//   }
// }

fn main() {
  run()
}