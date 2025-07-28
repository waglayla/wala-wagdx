use crate::imports::*;

pub struct Settings {
  #[allow(dead_code)]
  manager: DX_Manager,
  settings : crate::settings::Settings,
  wrpc_borsh_network_interface : NetworkInterfaceEditor,
  wrpc_json_network_interface : NetworkInterfaceEditor,
  grpc_network_interface : NetworkInterfaceEditor,
  reset_settings : bool,
}

impl Settings {
  pub fn new(manager: DX_Manager) -> Self {
    Self { 
      manager,
      settings : crate::settings::Settings::default(),
      wrpc_borsh_network_interface : NetworkInterfaceEditor::default(),
      wrpc_json_network_interface : NetworkInterfaceEditor::default(),
      grpc_network_interface : NetworkInterfaceEditor::default(),
      reset_settings : false,
    }
  }

  pub fn load(&mut self, settings : crate::settings::Settings) {
    self.settings = settings;

    self.wrpc_borsh_network_interface = NetworkInterfaceEditor::from(&self.settings.node.wrpc_borsh_network_interface);
    self.wrpc_json_network_interface = NetworkInterfaceEditor::from(&self.settings.node.wrpc_json_network_interface);
    self.grpc_network_interface = NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
  }

  pub fn render_node_storage_settings(_core: &mut Core, ui: &mut Ui, settings : &mut NodeSettings) -> Option<&'static str> {
    let mut node_settings_error = None;
    
    #[cfg(not(target_arch = "wasm32"))]
    {
      CollapsingHeader::new(i18n("Stratum Bridge"))
        .default_open(true)
        .show(ui, |ui| {
          ui.horizontal(|ui| {
            let response = ui.add(toggle(&mut settings.enable_bridge));
          });
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    if settings.node_kind.is_config_capable() {
      CollapsingHeader::new(i18n("Data Storage"))
        .default_open(true)
        .show(ui, |ui| {
          ui.checkbox(&mut settings.waglaylad_daemon_storage_folder_enable, i18n("Custom data storage folder"));
          if settings.waglaylad_daemon_args.contains("--appdir") && settings.waglaylad_daemon_storage_folder_enable {
            ui.colored_label(theme_color().warning_color, i18n("Your daemon arguments contain '--appdir' directive, which overrides the data storage folder setting."));
            ui.colored_label(theme_color().warning_color, i18n("Please remove the --appdir directive to continue."));
          } else if settings.waglaylad_daemon_storage_folder_enable {
            ui.horizontal(|ui|{
              ui.label(i18n("Data Storage Folder:"));
              ui.add(TextEdit::singleline(&mut settings.waglaylad_daemon_storage_folder));
            });

            let appdir = settings.waglaylad_daemon_storage_folder.trim();
            if appdir.is_empty() {
              ui.colored_label(theme_color().error_color, i18n("Data storage folder must not be empty"));
            } else if !Path::new(appdir).exists() {
              ui.colored_label(theme_color().error_color, i18n("Data storage folder not found at"));
              ui.label(format!("\"{}\"",settings.waglaylad_daemon_storage_folder.trim()));

              ui.add_space(4.);
              if ui.medium_button(i18n("Create Data Folder")).clicked() {
                if let Err(err) = std::fs::create_dir_all(appdir) {
                  manager().error(format!("Unable to create data storage folder `{appdir}`: {err}"));
                }
              }
              ui.add_space(4.);

              node_settings_error = Some(i18n("Data storage folder not found"));
            }
          }
        });
    }
    node_settings_error
  }

  pub fn render_remote_settings(_core: &mut Core, ui: &mut Ui, settings : &mut NodeSettings) -> Option<&'static str> {
    let mut node_settings_error = None;

    CollapsingHeader::new(i18n("Remote p2p Node Configuration"))
    .default_open(true)
    .show(ui, |ui| {

      ui.horizontal_wrapped(|ui|{
        ui.label(i18n("Remote Connection:"));
        NodeConnectionConfigKind::iter().for_each(|kind| {
            ui.radio_value(&mut settings.connection_config_kind, *kind, kind.to_string());
        });
      });

      match settings.connection_config_kind {
        NodeConnectionConfigKind::Custom => {
          CollapsingHeader::new(i18n("wRPC Connection Settings"))
            .default_open(true)
            .show(ui, |ui| {
              ui.horizontal(|ui|{
                ui.label(i18n("wRPC Encoding:"));
                WrpcEncoding::iter().for_each(|encoding| {
                  ui.radio_value(&mut settings.wrpc_encoding, *encoding, encoding.to_string());
                });
              });


              ui.horizontal(|ui|{
                ui.label(i18n("wRPC URL:"));
                ui.add(TextEdit::singleline(&mut settings.wrpc_url));
              });

              if let Err(err) = WaglaylaRpcClient::parse_url(settings.wrpc_url.clone(), settings.wrpc_encoding, Network::Mainnet.into()) {
                ui.label(
                  RichText::new(err.to_string())
                    .color(theme_color().warning_color),
                );
                node_settings_error = Some(i18n("Invalid wRPC URL"));
              }
            });
          },
          NodeConnectionConfigKind::PublicServerCustom => {
          },
          NodeConnectionConfigKind::PublicServerRandom => {
            ui.label(i18n("A random node will be selected on startup"));
          },
        }
      });

    node_settings_error
  }
}

impl ComponentT for Settings {
  fn init(&mut self, wallet : &mut Core) {
    self.load(wallet.settings.clone());
  }

  fn style(&self) -> ComponentStyle {
    ComponentStyle::Default
  }

  fn render(
    &mut self,
    core: &mut Core,
    _ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    ScrollArea::vertical()
      .auto_shrink([false, true])
      .show(ui, |ui| {
        self.render_settings(core,ui);
      });
  }

  fn deactivate(&mut self, _core: &mut Core) {
    #[cfg(not(target_arch = "wasm32"))]
    _core.storage.clear_settings();
  }
}

impl Settings {
  fn render_node_settings(
    &mut self,
    core: &mut Core,
    ui: &mut egui::Ui,
  ) {
      #[allow(unused_variables)]
      let half_width = ui.ctx().screen_rect().width() * 0.5;

      let mut node_settings_error = None;

      CollapsingHeader::new(i18n("WagLayla p2p Network & Node Connection"))
        .default_open(true)
        .show(ui, |ui| {
          CollapsingHeader::new(i18n("Node Mode"))
            .default_open(true)
            .show(ui, |ui| {
              ui.horizontal_wrapped(|ui|{
                WagLayladNodeKind::iter().for_each(|node_kind| {
                  let mut is_selected = self.settings.node.node_kind == *node_kind;
                  let response = ui.add(toggle(&mut is_selected));

                  if response.changed() {
                    self.settings.node.node_kind = *node_kind;
                  }
  
                  ui.label(node_kind.to_string());
                  response.on_hover_text_at_pointer(node_kind.describe());

                  ui.separator();
                });
              });

              match self.settings.node.node_kind {
                WagLayladNodeKind::Remote => {},
                WagLayladNodeKind::IntegratedAsDaemon => {
                  CollapsingHeader::new(i18n("Stratum Bridge"))
                    .default_open(true)
                    .show(ui, |ui| {
                      ui.horizontal(|ui| {
                        let response = ui.add(toggle(&mut self.settings.node.enable_bridge));
                      });
                    });
                },
                _ => { }
              }

              #[cfg(not(target_arch = "wasm32"))]
              if self.settings.node.node_kind.is_config_capable() {
                CollapsingHeader::new(i18n("Data Storage"))
                  .default_open(true)
                  .show(ui, |ui| {
                    ui.checkbox(&mut self.settings.node.waglaylad_daemon_storage_folder_enable, i18n("Custom data storage folder"));
                    if self.settings.node.waglaylad_daemon_args.contains("--appdir") && self.settings.node.waglaylad_daemon_storage_folder_enable {
                      ui.colored_label(theme_color().warning_color, i18n("Your daemon arguments contain '--appdir' directive, which overrides the data storage folder setting."));
                      ui.colored_label(theme_color().warning_color, i18n("Please remove the --appdir directive to continue."));
                    } else if self.settings.node.waglaylad_daemon_storage_folder_enable {
                      ui.horizontal(|ui|{
                        ui.label(i18n("Data Storage Folder:"));
                        ui.add(TextEdit::singleline(&mut self.settings.node.waglaylad_daemon_storage_folder));
                      });

                      let appdir = self.settings.node.waglaylad_daemon_storage_folder.trim();
                      if appdir.is_empty() {
                        ui.colored_label(theme_color().error_color, i18n("Data storage folder must not be empty"));
                      } else if !Path::new(appdir).exists() {
                        ui.colored_label(theme_color().error_color, i18n("Data storage folder not found at"));
                        ui.label(format!("\"{}\"",self.settings.node.waglaylad_daemon_storage_folder.trim()));

                        ui.add_space(4.);
                        if ui.medium_button(i18n("Create Data Folder")).clicked() {
                          if let Err(err) = std::fs::create_dir_all(appdir) {
                            manager().error(format!("Unable to create data storage folder `{appdir}`: {err}"));
                          }
                        }
                        ui.add_space(4.);

                        node_settings_error = Some(i18n("Data storage folder not found"));
                      }
                    }
                  });
              }
            });

        if !self.grpc_network_interface.is_valid() {
          node_settings_error = Some(i18n("Invalid gRPC network interface configuration"));
        } else {
          self.settings.node.grpc_network_interface = self.grpc_network_interface.as_ref().try_into().unwrap(); //NetworkInterfaceConfig::try_from(&self.grpc_network_interface).unwrap();
        }

        if self.settings.node.node_kind == WagLayladNodeKind::Remote {
          node_settings_error = Self::render_remote_settings(core, ui, &mut self.settings.node);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if self.settings.node.node_kind.is_config_capable() {
          CollapsingHeader::new(i18n("Local p2p Node Configuration"))
            .default_open(true)
            .show(ui, |ui| {
              ui.vertical(|ui|{
                CollapsingHeader::new(i18n("Client RPC"))
                  .default_open(true)
                  .show(ui, |ui| {
                    ui.vertical(|ui|{
                      ui.checkbox(&mut self.settings.node.enable_wrpc_borsh, i18n("Public wRPC (Borsh)"));

                      ui.checkbox(&mut self.settings.node.enable_grpc, i18n("Enable gRPC"));
                      if self.settings.node.enable_grpc {
                        CollapsingHeader::new(i18n("gRPC Network Interface & Port"))
                          .default_open(true)
                          .show(ui, |ui| {
                            self.grpc_network_interface.ui(ui);
                          });
                      }
                    });
              });
          
              CollapsingHeader::new(i18n("p2p RPC"))
                .default_open(true)
                .show(ui, |ui| {
                  ui.vertical(|ui|{
                    ui.checkbox(&mut self.settings.node.enable_upnp, i18n("Enable UPnP"));
                  });
                });
              });
            });
        }

      });

      if let Some(error) = node_settings_error {
        ui.add_space(4.);
        ui.label(
          RichText::new(error)
            .color(theme_color().error_color),
        );
        ui.add_space(4.);
        ui.label(i18n("Unable to change node settings until the problem is resolved"));

        if ui.button(i18n("Ok")).clicked() {
          self.settings.node = core.settings.node.clone();
          self.grpc_network_interface =
            NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
        }

        ui.separator();

      } else if node_settings_error.is_none() {
        if let Some(restart) = self.settings.node.compare(&core.settings.node) {
          if let Some(response) = ui.confirm_widget_labels("Apply", "Cancel") {
            match response {
              Confirm::Yes => {
                core.settings = self.settings.clone();
                core.settings.store_sync().unwrap();

                cfg_if! {
                  if #[cfg(not(target_arch = "wasm32"))] {
                    let storage_root = core.settings.node.waglaylad_daemon_storage_folder_enable.then_some(core.settings.node.waglaylad_daemon_storage_folder.as_str());
                    core.storage.track_storage_root(storage_root);
                  }
                }

                if restart {
                  self.manager.waglayla_service().update_services(&self.settings.node, None);
                  self.manager.bridge_service().update_services(&self.settings.node, None);
                }
              },
              Confirm::No => {
                self.settings = core.settings.clone();
                self.grpc_network_interface = NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
              }
            }
          }
          ui.separator();
        }
      }
  }

  fn render_bridge_settings(
    &mut self,
    core: &mut Core,
    ui: &mut egui::Ui,
  ) {
    if self.settings.node.node_kind != WagLayladNodeKind::IntegratedAsDaemon ||
      !self.settings.node.enable_bridge 
    {
      return;
    }

    let mut settings = &mut self.settings.bridge;
    let mut bridge_settings_error: Option<&str> = None;

    #[cfg(not(target_arch = "wasm32"))]
    CollapsingHeader::new(i18n("Stratum Bridge Config"))
      .default_open(false)
      .show(ui, |ui| {
      // Edit fields
      ui.horizontal(|ui| {
        ui.label("Stratum Port:");
        ui.text_edit_singleline(&mut settings.stratum_port)
          .on_hover_text_at_pointer(i18n(
            "The port that will be listening for incoming stratum traffic, preceded by a colon ':'."
          ));
      });
      ui.horizontal(|ui| {
        ui.label("Waglayla Address:");
        ui.text_edit_singleline(&mut settings.waglayla_address)
          .on_hover_text_at_pointer(i18n(
            "The <address>:<port> combo of an available WagLayla node, i.e 127.0.0.1:13110."
          ));
      });
      ui.horizontal(|ui| {
        ui.label("Min Share Diff:");
        ui.add(egui::DragValue::new(&mut settings.min_share_diff).speed(1))
          .on_hover_text_at_pointer(i18n(
            "Only accept shares of the specified difficulty (or higher) from the miner(s)."
          ));
      });
      ui.horizontal(|ui| {
        ui.add(toggle(&mut settings.var_diff))
          .on_hover_text_at_pointer(i18n(
            "Enables the auto-adjusting variable share difficulty mechanism. Allows for hashrate estimation."
          ));
        ui.label(i18n("Enable Vardiff"));
      });
      ui.horizontal(|ui| {
        ui.label("Shares Per Min:");
        ui.add(egui::DragValue::new(&mut settings.shares_per_min).speed(1))
          .on_hover_text_at_pointer(i18n(
            "The number of shares per minute the vardiff engine should target for every worker. Higher settings enable accurate hashrate estimation, at the cost of more network traffic."
          ));
      });
      // ui.horizontal(|ui| {
      //   ui.add(toggle(&mut settings.var_diff_stats));
      //   ui.label(i18n("Vardiff Stats"));
      // });
      ui.horizontal(|ui| {
        ui.add(toggle(&mut settings.solo_mining))
          .on_hover_text_at_pointer(i18n(
            "Disables stratum settings & vardiff calculations in favor of using the network difficulty directly."
          ));
        ui.label(i18n("Solo Mining Mode"));
      });
      ui.horizontal(|ui| {
        ui.label("Block Wait Time:");
        ui.text_edit_singleline(&mut settings.block_wait_time)
          .on_hover_text_at_pointer(i18n(
            "Amount of time to wait after a new block message from waglaylad before manually requesting a fresh template."
          ));
      });
      ui.horizontal(|ui| {
        ui.label("Extranonce Size:");
        ui.add(egui::DragValue::new(&mut settings.extranonce_size).range(0..=3))
          .on_hover_text_at_pointer(i18n(
            "Size in bytes for organizing worker jobs, from 0 (no extranonce) to 3. With no extranonce (0), all workers will search through the same nonce-space, deferring to miner-side randomization."
          ));
      });
      ui.horizontal(|ui| {
        ui.add(toggle(&mut settings.print_stats))
          .on_hover_text_at_pointer(i18n(
            "Enable printing stats to the console. Otherwise, only workers joining/disconnecting, blocks found, and errors will be printed."
          ));
        ui.label(i18n("Print Stats"));
      });
      ui.horizontal(|ui| {
        ui.add(toggle(&mut settings.log_to_file))
          .on_hover_text_at_pointer(i18n(
            "Write logs to a file local to the executable, in addition to the console."
          ));
        ui.label(i18n("Log to File"));
      });
      ui.horizontal(|ui| {
        ui.label("Prometheus Port:");
        ui.text_edit_singleline(&mut settings.prom_port)
          .on_hover_text_at_pointer(i18n(
            "Prometheus will serve stats on the port provided."
          ));
      });

      if let Some(error) = bridge_settings_error {
        ui.add_space(4.);
        ui.label(
          RichText::new(error)
            .color(theme_color().error_color),
        );
        ui.add_space(4.);
        ui.label(i18n("Unable to change node settings until the problem is resolved"));

        if ui.button(i18n("Ok")).clicked() {
          *settings = core.settings.bridge.clone();
          self.grpc_network_interface =
            NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
        }

        ui.separator();

      } else if bridge_settings_error.is_none() {
        if *settings != core.settings.bridge {
          if let Some(response) = ui.confirm_widget_labels("Apply", "Cancel") {
            match response {
              Confirm::Yes => {
                core.settings.bridge = settings.clone();
                core.settings.store_sync().unwrap();

                cfg_if! {
                  if #[cfg(not(target_arch = "wasm32"))] {
                    let storage_root = core.settings.node.waglaylad_daemon_storage_folder_enable.then_some(core.settings.node.waglaylad_daemon_storage_folder.as_str());
                    core.storage.track_storage_root(storage_root);
                  }
                }

                self.manager.stop_services();
                self.manager.start_services();
              },
              Confirm::No => {
                *settings = core.settings.bridge.clone();
              }
            }
          }
          ui.separator();
        }
      }
    });
  }

  fn render_ui_settings(
    &mut self,
    core: &mut Core,
    ui: &mut egui::Ui,
  ) {
    CollapsingHeader::new(i18n("User Interface"))
      .default_open(false)
      .show(ui, |ui| {

        ui.horizontal(|ui| {
          ui.add(toggle(&mut core.settings.user_interface.enable_sfx))
            .on_hover_text_at_pointer(i18n("Turn Layla's barking on/off"));
          ui.label(i18n("Sound Effects"));
        });
        ui.horizontal(|ui| {
          ui.add(toggle(&mut core.settings.user_interface.show_coinbase))
            .on_hover_text_at_pointer(i18n("Show notifications for newly minted coins from mining blocks"));
          ui.label(i18n("Coinbase Notifications"));
        });

        CollapsingHeader::new(i18n("Theme"))
          .default_open(true)
          .show(ui, |ui| {
            ui.vertical(|ui| {
              ui.horizontal(|ui| {
                let current_theme_color_name = theme_color().name();
                ui.menu_button(
                  format!("{} ⏷", current_theme_color_name),
                  |ui| {
                    theme_colors().keys().for_each(|name| {
                      if name.as_str() != current_theme_color_name
                        && ui.add_sized(
                          [65.0, 16.0],
                          egui::Button::new(name)
                      ).clicked()
                      {
                        apply_theme_color_by_name(
                          ui.ctx(),
                          name,
                        );
                        core
                          .settings
                          .user_interface
                          .theme_color = name.to_string();
                        core.store_settings();
                        ui.close_menu();
                      }
                    });
                  },
                );
              });
            });

            ui.add_space(1.);
          });

          if workflow_core::runtime::is_native() {
            CollapsingHeader::new(i18n("Zoom"))
              .default_open(true)
              .show(ui, |ui| {
                ui.horizontal(|ui| {
                  let zoom_factor = ui.ctx().zoom_factor();
                  if ui
                    .add_sized(
                      Vec2::splat(24.),
                      Button::new(RichText::new("-").size(18.)),
                    )
                    .clicked()
                  {
                    ui.ctx().set_zoom_factor(zoom_factor - 0.1);
                  }
                  ui.label(format!("{:.0}%", zoom_factor * 100.0));
                  if ui
                    .add_sized(
                      Vec2::splat(24.),
                      Button::new(RichText::new("+").size(18.)),
                    )
                    .clicked()
                  {
                    ui.ctx().set_zoom_factor(zoom_factor + 0.1);
                  }
                });

                ui.add_space(1.);
              });
          }
      });
  }

  fn render_settings(
    &mut self,
    core: &mut Core,
    ui: &mut egui::Ui,
  ) {

    self.render_ui_settings(core,ui);
    self.render_node_settings(core,ui);
    self.render_bridge_settings(core,ui);
        
    #[cfg(not(target_arch = "wasm32"))]
    core.storage.clone().render_settings(core, ui);
  }
}
