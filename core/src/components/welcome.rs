use crate::imports::*;
use crate::components::hello::Hello;

pub struct Welcome {
    #[allow(dead_code)]
    manager: DXManager,
    settings : Settings,
}

impl Welcome {
    pub fn new(manager: DXManager) -> Self {

        #[allow(unused_mut)]
        let mut settings = Settings::default();

        #[cfg(target_arch = "wasm32")] {
            settings.node.node_kind = WaglayladNodeKind::IntegratedAsDaemon;
        }

        Self { 
            manager, 
            settings,
        }
    }

    pub fn render_native(
        &mut self,
        core: &mut Core,
        ui: &mut egui::Ui,
    ) {

        let mut error = None;

        ui.heading(i18n("Welcome to Waglayla Wag-DX"));
        ui.add_space(16.0);
        ui.label(i18n("Please configure your Waglayla Wag-DX settings"));
        ui.add_space(16.0);

        CollapsingHeader::new(i18n("Settings"))
            .default_open(true)
            .show(ui, |ui| {
                CollapsingHeader::new(i18n("Waglayla p2p Node & Connection"))
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            // WaglayladNodeKind::iter().for_each(|node| {
                            [
                                WaglayladNodeKind::Disabled,
                                WaglayladNodeKind::Remote,
                                #[cfg(not(target_arch = "wasm32"))]
                                WaglayladNodeKind::IntegratedAsDaemon,
                            ].iter().for_each(|node_kind| {
                                let mut is_selected = self.settings.node.node_kind == *node_kind;
                                let response = ui.add(toggle(&mut is_selected));
                                
                                if response.changed() {
                                    self.settings.node.node_kind = *node_kind;
                                }
                
                                ui.label(node_kind.to_string());
                                response.on_hover_text_at_pointer(node_kind.describe());
                            });
                        });

                        if self.settings.node.node_kind == WaglayladNodeKind::Remote {
                            error = crate::components::settings::Settings::render_remote_settings(core,ui,&mut self.settings.node);
                        } else if self.settings.node.node_kind == WaglayladNodeKind::IntegratedAsDaemon {
                            error = crate::components::settings::Settings::render_node_storage_settings(core,ui,&mut self.settings.node);
                        }
                    });

                CollapsingHeader::new(i18n("User Interface"))
                    .default_open(true)
                    .show(ui, |ui| {

                        ui.horizontal(|ui| {

                            ui.label(i18n("Language:"));

                            let language_code = core.settings.language_code.clone();
                            let dictionary = i18n::dictionary();
                            let language = dictionary.language_title(language_code.as_str()).unwrap();//.unwrap();
                            egui::ComboBox::from_id_source("language_selector")
                                .selected_text(language)
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(60.0);
                                    dictionary.enabled_languages().into_iter().for_each(|(code,lang)| {
                                        ui.selectable_value(&mut self.settings.language_code, code.to_string(), lang);
                                    });
                                });

                            ui.add_space(16.);
                            ui.label(i18n("Theme Color:"));

                            let mut theme_color = self.settings.user_interface.theme_color.clone();
                            egui::ComboBox::from_id_source("theme_color_selector")
                                .selected_text(theme_color.as_str())
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(60.0);
                                    theme_colors().keys().for_each(|name| {
                                        ui.selectable_value(&mut theme_color, name.to_string(), name);
                                    });
                                });
                                
                            if theme_color != self.settings.user_interface.theme_color {
                                self.settings.user_interface.theme_color = theme_color;
                                apply_theme_color_by_name(ui.ctx(), self.settings.user_interface.theme_color.clone());
                            }

                            ui.add_space(16.);
                            ui.label(i18n("Theme Style:"));

                            let mut theme_style = self.settings.user_interface.theme_style.clone();
                            egui::ComboBox::from_id_source("theme_style_selector")
                                .selected_text(theme_style.as_str())
                                .show_ui(ui, |ui| {
                                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                                    ui.set_min_width(60.0);
                                    theme_styles().keys().for_each(|name| {
                                        ui.selectable_value(&mut theme_style, name.to_string(), name);
                                    });
                                });
                                
                            if theme_style != self.settings.user_interface.theme_style {
                                self.settings.user_interface.theme_style = theme_style;
                                apply_theme_style_by_name(ui.ctx(), self.settings.user_interface.theme_style.clone());
                            }
                        });        
                    });

                ui.add_space(32.0);
                if let Some(error) = error {
                    ui.vertical_centered(|ui| {
                        ui.colored_label(theme_color().alert_color, error);
                    });
                    ui.add_space(32.0);
                } else {
                    
                    ui.horizontal(|ui| {
                        ui.add_space(
                            ui.available_width()
                                - 16.
                                - (theme_style().medium_button_size.x + ui.spacing().item_spacing.x),
                        );
                        if ui.medium_button(format!("{} {}", egui_phosphor::light::CHECK, i18n("Apply"))).clicked() {
                            let mut settings = self.settings.clone();
                            settings.initialized = true;
                            let message = i18n("Unable to store settings");
                            settings.store_sync().expect(message);
                            self.manager.waglayla_service().update_services(&self.settings.node, None);
                            core.settings = settings.clone();
                            core.get_mut::<components::settings::Settings>().load(settings);
                            core.set_active_component_by_type(TypeId::of::<Hello>());
                        }
                    });
                }

                ui.separator();
        });
        
        ui.vertical_centered(|ui| {
            ui.add_space(32.0);
            // ui.colored_label(theme_color().alert_color, "Please note - this is a beta release - Waglayla Wag-DX is still in early development and is not yet ready for production use.");
            // ui.add_space(32.0);
            ui.label(format!("Waglayla Wag-DX v{}  •  Rusty Waglayla v{}", env!("CARGO_PKG_VERSION"), waglayla_wallet_core::version()));
            ui.hyperlink_to(
                "https://waglayla.org",
                "https://waglayla.org",
            );
    
        });
    }

    // pub fn render_web(
    //     &mut self,
    //     core: &mut Core,
    //     ui: &mut egui::Ui,
    // ) {
    //     let mut proceed = false;

    //     Panel::new(self)
    //         .with_caption(i18n("Welcome to Waglayla Wag-DX"))
    //         .with_header(|_this, ui| {
    //             ui.label(i18n("Please select Waglayla network"));
    //         })
    //         .with_body(|this, ui| {
    //             Network::iter().for_each(|network| {
    //                 if ui.add_sized(
    //                         theme_style().large_button_size,
    //                         CompositeButton::opt_image_and_text(
    //                             None,
    //                             Some(network.name().into()),
    //                             Some(network.describe().into()),
    //                         ),
    //                     )
    //                     .clicked()
    //                 {
    //                     this.settings.node.network = *network;
    //                     proceed = true;
    //                 }

    //                 ui.add_space(8.);
    //             });

    //             ui.add_space(32.0);
                
    //             ui.colored_label(theme_color().alert_color, RichText::new("β").size(64.0));
    //             // ui.add_space(8.0);
    //             // ui.colored_label(theme_color().alert_color, "Please note - this is a beta release - Waglayla Wag-DX is still in early development and is not yet ready for production use.");
    //         })
    //         .render(ui);        

    //     if proceed {
    //         let mut settings = self.settings.clone();
    //         settings.initialized = true;
    //         let message = i18n("Unable to store settings");
    //         settings.store_sync().expect(message);
    //         core.settings = settings.clone();
    //         self.manager.waglayla_service().update_services(&settings.node, None);

    //         core.get_mut::<components::settings::Settings>().load(settings);
    //         core.set_active_component_by_type(TypeId::of::<Hello>());
    //     }

    // }

}

impl ComponentT for Welcome {

    fn style(&self) -> ComponentStyle {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                ComponentStyle::Mobile
            } else {
                ComponentStyle::Default
            }
        }
    }

    fn render(
        &mut self,
        core: &mut Core,
        _ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        ui: &mut egui::Ui,
    ) {
        cfg_if! {
            if #[cfg(not(target_arch = "wasm32"))] {
                self.render_native(core, ui)
            } else {
                self.render_web(core, ui)
            }
        }
    }

}