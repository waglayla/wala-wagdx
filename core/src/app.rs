use crate::imports::*;
use crate::events::ApplicationEventsChannel;
use crate::result::Result;
use cfg_if::cfg_if;

use eframe::HardwareAcceleration;

use wala_wagdx_core::dx_manager;
use wala_wagdx_core::settings::Settings;
use waglayla_wallet_core::api::WalletApi;

use eframe::Renderer;

use std::sync::Arc;
use workflow_i18n::*;
use workflow_log::*;

pub const WALA_WAGDX_ICON_PNG: &[u8] = include_bytes!("../resources/icons/icon-256.png");

pub const I18N_EMBEDDED: &str = include_str!("../resources/i18n/i18n.json");
pub const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");
pub const GIT_DESCRIBE: &str = env!("VERGEN_GIT_DESCRIBE");
pub const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
pub const RUSTC_CHANNEL: &str = env!("VERGEN_RUSTC_CHANNEL");
pub const RUSTC_COMMIT_DATE: &str = env!("VERGEN_RUSTC_COMMIT_DATE");
pub const RUSTC_COMMIT_HASH: &str = env!("VERGEN_RUSTC_COMMIT_HASH");
pub const RUSTC_HOST_TRIPLE: &str = env!("VERGEN_RUSTC_HOST_TRIPLE");
pub const RUSTC_LLVM_VERSION: &str = env!("VERGEN_RUSTC_LLVM_VERSION");
pub const RUSTC_SEMVER: &str = env!("VERGEN_RUSTC_SEMVER");
pub const CARGO_TARGET_TRIPLE: &str = env!("VERGEN_CARGO_TARGET_TRIPLE");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Clone)]
pub struct ApplicationContext {
  pub wallet_api: Option<Arc<dyn WalletApi>>,
}

impl ApplicationContext {
  pub fn new(
    wallet_api: Option<Arc<dyn WalletApi>>,
  ) -> Self {
    Self {
      wallet_api,
    }
  }
}

cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    use waglaylad_lib::daemon::{
      create_core,
    };
    use waglaylad_lib::args::Args as NodeArgs;
    use waglayla_utils::fd_budget;
    use waglayla_core::signals::Signals;
    use clap::ArgAction;
    use crate::utils::*;
    use std::fs;

    const DESIRED_DAEMON_SOFT_FD_LIMIT: u64 = 4 * 1024;
    const MINIMUM_DAEMON_SOFT_FD_LIMIT: u64 = 2 * 1024;

    #[derive(Debug)]
    enum I18n {
      Import,
      Export,
      Reset,
    }

    enum Args {
      I18n { op : I18n },
      Wdx {
        reset_settings : bool,
      },
      WagLaylad { args : Box<NodeArgs> },
    }

    fn include_icon(png_bytes: &[u8]) -> std::result::Result<IconData, Box<dyn std::error::Error>> {  
      let image = image::load_from_memory(png_bytes)?;
      let rgba = image.into_rgba8().into_raw();
  
      Ok(IconData { rgba, height: 256, width: 256 })
    }

    // TODO: re-engineer from scratch based on determined needs
    fn parse_args() -> Args {
      #[allow(unused)]
      use clap::{arg, command, Arg, Command};
      use std::env::{args,var};
      use std::iter::once;

      if args().any(|arg| arg == "--daemon") || var("WALA_WAGDX_DAEMON").is_ok() {
        let args = once("waglaylad".to_string()).chain(args().skip(1).filter(|arg| arg != "--daemon"));//.collect::<Vec<String>>();
        match NodeArgs::parse(args) {
          Ok(args) => Args::WagLaylad { args : Box::new(args) },
          Err(err) => {
            println!("{err}");
            std::process::exit(1);
          }
        }
      } else {
        let cmd = Command::new("wala-wagdx")
          .about(format!("wala-wagdx v{VERSION}-{GIT_DESCRIBE} (rusty-waglayla {})",  waglayla_wallet_core::version()))
          .arg(arg!(--version "Display software version"))
          .arg(arg!(--daemon "Run as WagLayla Rusty p2p daemon"))
          .arg(
            Arg::new("reset-settings")
            .long("reset-settings")
            .action(ArgAction::SetTrue)
            .help("Reset wala-wagdx settings")
          )
          .subcommand(
            Command::new("i18n").hide(true)
            .about("wala-wagdx i18n user interface translation")
            .subcommand(
              Command::new("import")
              .about("import JSON files suffixed with language codes (*_en.json, *_de.json, etc.)")
            )
            .subcommand(
              Command::new("export")
              .about("export default 'en' translations as JSON")
            )
            .subcommand(
              Command::new("reset")
              .about("reset i18n data file")
            )
          );

        let matches = cmd.get_matches();

        if matches.get_one::<bool>("version").cloned().unwrap_or(false) {
          std::process::exit(0);
        } else if let Some(matches) = matches.subcommand_matches("i18n") {
          if let Some(_matches) = matches.subcommand_matches("import") {
            Args::I18n { op : I18n::Import }
          } else if let Some(_matches) = matches.subcommand_matches("export") {
            Args::I18n { op : I18n::Export }
          } else if let Some(_matches) = matches.subcommand_matches("reset") {
            Args::I18n { op : I18n::Reset }
          } else {
            println!();
            println!("please specify a valid i18n subcommand");
            std::process::exit(1);
          }
        } else {
          let reset_settings = matches.get_one::<bool>("reset-settings").cloned().unwrap_or(false);

          Args::Wdx { reset_settings }
        }
      }
    }

    pub async fn wagdx_main(application_context : ApplicationContext) -> Result<()> {
      use std::sync::Mutex;

      let ApplicationContext { wallet_api } = application_context;

      match try_set_fd_limit(DESIRED_DAEMON_SOFT_FD_LIMIT) {
        Ok(limit) => {
          if limit < MINIMUM_DAEMON_SOFT_FD_LIMIT {
            println!();
            println!("| Current OS file descriptor limit (soft FD limit) is set to {limit}");
            println!("| The waglaylad node requires a setting of at least {DESIRED_DAEMON_SOFT_FD_LIMIT} to operate properly.");
            println!("| Please increase the limits using the following command:");
            println!("| ulimit -n {DESIRED_DAEMON_SOFT_FD_LIMIT}");
            println!();
          }
        }
        Err(err) => {
          println!();
          println!("| Unable to initialize the necessary OS file descriptor limit (soft FD limit) to: {}", err);
          println!("| The waglaylad node requires a setting of at least {DESIRED_DAEMON_SOFT_FD_LIMIT} to operate properly.");
          println!();
        }
      }

      match parse_args() {
        Args::WagLaylad{ args } => {
          init_ungraceful_panic_handler();
          let fd_total_budget = fd_budget::limit() - args.rpc_max_clients as i32 - args.inbound_limit as i32 - args.outbound_target as i32;
          let (core, _) = create_core(*args, fd_total_budget);
          Arc::new(Signals::new(&core)).init();
          core.run();
        }

        Args::I18n {
          op
        } => {
          init_ungraceful_panic_handler();
          manage_i18n(op)?;
        }

        Args::Wdx { reset_settings } => {
          workflow_log::set_colors_enabled(true);
          println!("wala-wagdx v{VERSION}-{GIT_DESCRIBE} (rusty-waglayla {})", waglayla_wallet_core::version());

          env_logger::init();

          set_log_level(LevelFilter::Info);

          let mut settings = if reset_settings {
            println!("Resetting wala-wagdx settings on user request...");
            Settings::default().store_sync()?.clone()
          } else {
            Settings::load().await.unwrap_or_else(|err| {
              log_error!("Unable to load settings: {err}");
              Settings::default()
            })
          };

          let i18n_json_file = i18n_storage_file()?;
          let i18n_json_file_load = i18n_json_file.clone();
          let i18n_json_file_store = i18n_json_file.clone();
          
          i18n::Builder::new(settings.language_code.as_str(), "en")
            .with_static_json_data(I18N_EMBEDDED)
            .with_string_json_data(i18n_json_file.exists().then(move ||{
              fs::read_to_string(i18n_json_file_load)
            }).transpose()?)
            .with_store(move |json_data: &str| {
              Ok(fs::write(&i18n_json_file_store, json_data)?)
            })
            .try_init()?;

          let manager: Arc<Mutex<Option<dx_manager::DX_Manager>>> = Arc::new(Mutex::new(None));
          let delegate = manager.clone();

          let icon_data = include_icon(WALA_WAGDX_ICON_PNG).expect("Failed to load embedded icon");

          let mut viewport = egui::ViewportBuilder::default()
            .with_title(i18n("WagLayla Wag-DX"))
            .with_min_inner_size([1030.0, 600.0])
            .with_inner_size([1030.0,600.0])
            .with_icon(icon_data)
            .with_decorations(false) // For window frame
            .with_transparent(true) // For window frame
            .with_resizable(true)
          ;
  
          let native_options = {
            #[cfg(target_os = "macos")]
            {
              eframe::NativeOptions {
                persist_window: true,
                viewport,
                renderer: eframe::Renderer::Glow,
                hardware_acceleration: HardwareAcceleration::Preferred,
                vsync: true,
                ..Default::default()
              }
            }
            
            #[cfg(not(target_os = "macos"))]
            {
              eframe::NativeOptions {
                persist_window: true,
                viewport,
                renderer: eframe::Renderer::Glow,
                hardware_acceleration: HardwareAcceleration::Preferred,
                vsync: true,
                ..Default::default()
              }
            }
          };

          let application_events = ApplicationEventsChannel::unbounded();
          let daemon_channel = Channel::<DaemonMessage>::unbounded();
          
          eframe::run_native(
            "WagDX",
            native_options,
            Box::new(move |cc| {
              Assets::init(&cc.egui_ctx);
              let manager = dx_manager::DX_Manager::new(
                &cc.egui_ctx, 
                Some(application_events), 
                wallet_api, 
                &settings, 
                daemon_channel.clone()
              );
              
              delegate.lock().unwrap().replace(manager.clone());
              dx_manager::signal_handler::Signals::bind(&manager);
              manager.start();
  
              Ok(Box::new(wala_wagdx_core::Core::new(cc, settings, true, daemon_channel.receiver.clone())))
            }),
          )?;
          let manager = manager.lock().unwrap().take().unwrap();
          manager.shutdown().await;
        }
      }

      Ok(())
    }
  } else {
    pub async fn wagdx_main(application_context : ApplicationContext) -> Result<()> {
      println("WASM target is not implemented");
      std::process::exit(1);
    }
  }
}

cfg_if! {
  if #[cfg(not(target_arch = "wasm32"))] {
    fn manage_i18n(op : I18n) -> Result<()> {
      if matches!(op, I18n::Reset) {
        println!("resetting i18n data file");
        i18n::create(i18n_storage_file()?)?;
        return Ok(());
      }

      let i18n_json_file = i18n_storage_file()?;
      let i18n_json_file_store = i18n_storage_file()?;
      i18n::Builder::new("en", "en")
        .with_static_json_data(I18N_EMBEDDED)
        .with_string_json_data(i18n_json_file.exists().then(move ||{
          fs::read_to_string(i18n_json_file)
        }).transpose()?)
        .with_store(move |json_data: &str| {
          Ok(fs::write(&i18n_json_file_store, json_data)?)
        })
        .try_init()?;

      match op {
      I18n::Import => {
          let source_folder = i18n_storage_folder()?;
          println!("importing translation files from: '{}'", source_folder.display());
          i18n::import_translation_files(source_folder,false)?;
        }
        I18n::Export => {
          let mut target_folder = if let Some(cwd) = try_cwd_repo_root()? {
            cwd.join("resources").join("i18n")
          } else {
            std::env::current_dir()?
          };
          target_folder.push("waglayla-ng_en.json");
          println!("exporting default language to: '{}'", target_folder.display());
          i18n::export_default_language(move |json_data: &str| {
            Ok(fs::write(&target_folder, json_data)?)
          })?;
        }
        _ => unreachable!()
      }

      Ok(())
    }
  }
}


#[cfg(not(target_arch = "wasm32"))]
pub fn try_set_fd_limit(limit: u64) -> Result<u64> {
  cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
      Ok(rlimit::setmaxstdio(limit as u32).map(|v| v as u64)?)
    } else if #[cfg(unix)] {
      Ok(rlimit::increase_nofile_limit(limit)?)
    }
  }
}

use std::panic;

pub fn init_ungraceful_panic_handler() {
  let default_hook = panic::take_hook();
  panic::set_hook(Box::new(move |panic_info| {
    default_hook(panic_info);
    println!("Exiting...");
    std::process::exit(1);
  }));
}
