use crate::imports::*;

pub struct Settings {
    #[allow(dead_code)]
    manager: DXManager,
    settings : crate::settings::Settings,
    wrpc_borsh_network_interface : NetworkInterfaceEditor,
    wrpc_json_network_interface : NetworkInterfaceEditor,
    grpc_network_interface : NetworkInterfaceEditor,
    reset_settings : bool,
}

impl Settings {
    pub fn new(manager: DXManager) -> Self {
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

    // pub fn change_current_network(&mut self, network : Network) {
    //     self.settings.node.network = network;
    // }

    pub fn render_node_storage_settings(_core: &mut Core, ui: &mut Ui, settings : &mut NodeSettings) -> Option<&'static str> {
      let mut node_settings_error = None;
      
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
                    // cfg_if! {
                    //     if #[cfg(not(target_arch = "wasm32"))] {
                    //         ui.horizontal_wrapped(|ui|{
                    //             ui.label(i18n("Recommended arguments for the remote node: "));
                    //             ui.label(RichText::new("waglaylad --utxoindex --rpclisten-borsh=0.0.0.0").code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                    //         });
                    //         ui.horizontal_wrapped(|ui|{
                    //             ui.label(i18n("If you are running locally, use: "));
                    //             ui.label(RichText::new("--rpclisten-borsh=127.0.0.1.").code().font(FontId::monospace(14.0)).color(theme_color().strong_color));
                    //         });
                    //     }
                    // }

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

        CollapsingHeader::new(i18n("Waglayla p2p Network & Node Connection"))
            .default_open(true)
            .show(ui, |ui| {

                CollapsingHeader::new(i18n("Waglayla Node"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui|{
                            WaglayladNodeKind::iter().for_each(|node_kind| {
                                ui.radio_value(&mut self.settings.node.node_kind, *node_kind, node_kind.to_string()).on_hover_text_at_pointer(node_kind.describe());
                            });
                        });

                        match self.settings.node.node_kind {
                            WaglayladNodeKind::Remote => {

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

                if self.settings.node.node_kind == WaglayladNodeKind::Remote {
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

                                            // ui.checkbox(&mut self.settings.node.enable_wrpc_json, i18n("Enable wRPC JSON"));
                                            // if self.settings.node.enable_wrpc_json {
                                            //     CollapsingHeader::new(i18n("wRPC JSON Network Interface & Port"))
                                            //         .default_open(true)
                                            //         .show(ui, |ui| {
                                            //             self.wrpc_json_network_interface.ui(ui);
                                            //         });
                                            // }

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
                            // });
                            
                                CollapsingHeader::new(i18n("p2p RPC"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.vertical(|ui|{
                                            ui.checkbox(&mut self.settings.node.enable_upnp, i18n("Enable UPnP"));
                                        });
                                    });
                                });
                        });
                } // is_config_capable

            }); // Waglayla p2p Network & Node Connection

            if let Some(error) = node_settings_error {
                ui.add_space(4.);
                ui.label(
                    RichText::new(error)
                        .color(theme_color().error_color),
                );
                ui.add_space(4.);
                ui.label(i18n("Unable to change node settings until the problem is resolved"));

                ui.add_space(8.);

                if ui.button(i18n("Ok")).clicked() {
                    self.settings.node = core.settings.node.clone();
                    self.grpc_network_interface =
                        NetworkInterfaceEditor::from(&self.settings.node.grpc_network_interface);
                }

                ui.separator();

            } else if node_settings_error.is_none() {
                if let Some(restart) = self.settings.node.compare(&core.settings.node) {

                    ui.add_space(16.);
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




    fn render_ui_settings(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {


        CollapsingHeader::new(i18n("User Interface"))
            .default_open(false)
            .show(ui, |ui| {

                CollapsingHeader::new(i18n("Theme Color"))
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
                                                && ui.button(name).clicked()
                                            {
                                                apply_theme_color_by_name(
                                                    ui.ctx(),
                                                    name,
                                                );
                                                core
                                                    .settings
                                                    .user_interface
                                                    .theme_color = name.to_string();
                                                // core.store_settings();
                                                ui.close_menu();
                                            }
                                        });
                                    },
                                );
                            });
                        });

                        ui.add_space(1.);
                    });

                    CollapsingHeader::new(i18n("Theme Style"))
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let current_theme_style_name = theme_style().name();
                                ui.menu_button(
                                    format!("{} ⏷", current_theme_style_name),
                                    |ui| {
                                        theme_styles().keys().for_each(|name| {
                                            if name.as_str() != current_theme_style_name
                                                && ui.button(name).clicked()
                                            {
                                                apply_theme_style_by_name(ui.ctx(), name);
                                                core
                                                    .settings
                                                    .user_interface
                                                    .theme_style = name.to_string();
                                                // core.store_settings();
                                                ui.close_menu();
                                            }
                                        });
                                    },
                                );
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

        self.render_node_settings(core,ui);

        self.render_ui_settings(core,ui);

        // CollapsingHeader::new(i18n("Services"))
        //     .default_open(true)
        //     .show(ui, |ui| {

        //         CollapsingHeader::new(i18n("Market Monitor"))
        //             .default_open(true)
        //             .show(ui, |ui| {
        //                 if ui.checkbox(&mut self.settings.market_monitor, i18n("Enable Market Monitor")).changed() {
        //                     core.settings.market_monitor = self.settings.market_monitor;
        //                     self.manager.market_monitor_service().enable(core.settings.market_monitor);
        //                     core.store_settings();
        //                 }
        //             });

        //         #[cfg(not(target_arch = "wasm32"))]
        //         CollapsingHeader::new(i18n("Check for Updates"))
        //             .default_open(true)
        //             .show(ui, |ui| {
        //                 if ui.checkbox(&mut self.settings.update_monitor, i18n("Check for Software Updates on GitHub")).changed() {
        //                     core.settings.update_monitor = self.settings.update_monitor;
        //                     self.manager.update_monitor_service().enable(core.settings.update_monitor);
        //                     core.store_settings();
        //                 }
        //             });    
        //     });

        // CollapsingHeader::new(i18n("Network Fee Estimator"))
        //     .default_open(false)
        //     .show(ui, |ui| {
        //         ui.vertical(|ui|{
        //             EstimatorMode::iter().for_each(|kind| {
        //                 ui.radio_value(&mut self.settings.estimator.mode, *kind, i18n(kind.describe()));
        //             });
                    
        //             if self.settings.estimator.mode != core.settings.estimator.mode {
        //                 core.settings.estimator.mode = self.settings.estimator.mode;
        //                 core.store_settings();
        //             }
        //         });
        //     });
            
        #[cfg(not(target_arch = "wasm32"))]
        core.storage.clone().render_settings(core, ui);

        // CollapsingHeader::new(i18n("Advanced"))
        //     .default_open(false)
        //     .show(ui, |ui| {
        //         if !self.reset_settings {
        //             ui.vertical(|ui|{
        //                 ui.set_max_width(340.);
        //                 ui.separator();
        //                 if ui.medium_button(i18n("Reset Settings")).clicked() {
        //                     self.reset_settings = true;
        //                 }
        //             });
        //         } else {
        //             ui.add_space(16.);
        //             ui.label(RichText::new(i18n("Are you sure you want to reset all settings?")).color(theme_color().warning_color));
        //             ui.add_space(16.);
        //             if let Some(response) = ui.confirm_medium_apply_cancel(Align::Min) {
        //                 match response {
        //                     Confirm::Ack => {
        //                         let settings = crate::settings::Settings {
        //                             initialized : true,
        //                             ..Default::default()
        //                         };
        //                         self.settings = settings.clone();
        //                         settings.store_sync().unwrap();
        //                         #[cfg(target_arch = "wasm32")]
        //                         workflow_dom::utils::window().location().reload().ok();
        //                     },
        //                     Confirm::Nack => {
        //                         self.reset_settings = false;
        //                     }
        //                 }
        //             }
        //             ui.separator();
        //         }
        //     });
    }
}