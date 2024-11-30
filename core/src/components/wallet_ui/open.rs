use crate::imports::*;
use waglayla_wallet_core::{wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, encryption::EncryptionKind, api::{AccountsDiscoveryRequest, AccountsDiscoveryKind}};
use waglayla_bip32::{WordCount, Mnemonic, Language};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[derive(Clone, Default)]
pub enum State {
  #[default]
  Select,
  KeySelection,
  SetNames,
  PhishingHint,
  WalletSecrets,
  Create { is_pending: bool, result: Arc<Mutex<Option<Result<(String, AccountDescriptor)>>>> },
  PresentSeed,
  Unlock { wallet_descriptor : WalletDescriptor, error : Option<Arc<Error>>},
  Unlocking { wallet_descriptor : WalletDescriptor },
}

#[derive(Clone, Default)]
struct CreationArgs {
  word_count : WordCount,
  wallet_name: String,
  wallet_filename: String,
  account_name: String,
  enable_phishing_hint: bool,
  phishing_hint: String,
  wallet_secret: String,
  decrypt_wallet_secret: String,
  wallet_secret_confirm: String,
  wallet_secret_show: bool,
  wallet_secret_score: Option<f64>,
  enable_payment_secret: bool,
  payment_secret: String,
  payment_secret_confirm: String,
  payment_secret_show : bool,
  payment_secret_score: Option<f64>,
  // mnemonic_presenter_context : MnemonicPresenterContext,
  import_private_key : bool,
  import_private_key_file: bool,
  import_private_key_mnemonic : String,
  import_private_key_mnemonic_error : Option<String>,
  import_with_bip39_passphrase : bool,
  import_legacy : bool,
  import_advanced : bool,

  show_confirmation_popup: bool,
  show_secrets_in_popup: bool,
  popup_background_opacity: f32,
  // wallet_file_data: Option<WalletFileData>
}

impl Zeroize for CreationArgs {
  fn zeroize(&mut self) {
    self.wallet_name.zeroize();
    self.wallet_filename.zeroize();
    self.account_name.zeroize();
    self.phishing_hint.zeroize();
    self.wallet_secret.zeroize();
    self.wallet_secret_confirm.zeroize();
    self.payment_secret.zeroize();
    self.payment_secret_confirm.zeroize();
    // self.mnemonic_presenter_context.zeroize();

    self.import_private_key.zeroize();
    self.import_private_key_mnemonic.zeroize();
    self.import_private_key_mnemonic_error.zeroize();
    self.import_with_bip39_passphrase.zeroize();
    self.decrypt_wallet_secret.zeroize();
    self.import_legacy.zeroize();
    self.import_advanced.zeroize();
    self.show_confirmation_popup = false;
    self.show_secrets_in_popup = false;
  }
}

pub struct OpenWallet {
  #[allow(dead_code)]
  manager: DXManager,
  wallet_secret: String,
  args_create: CreationArgs,
  pub state: State,
  pub message: Option<String>,
}

impl OpenWallet {
  pub fn new(manager: DXManager) -> Self {
    Self {
      manager,
      wallet_secret: String::new(),
      args_create: Default::default(),
      state: State::Select,
      message: None,
    }
  }

  pub fn open(&mut self, wallet_descriptor: WalletDescriptor) {
    self.state = State::Unlock { wallet_descriptor, error : None};
  }
}

enum WizardAction {
  Back,
  Next(State),
  NoAction,
}

fn render_centered_content<F>(ui: &mut egui::Ui, title: &str, content: F) -> WizardAction
where
    F: FnOnce(&mut egui::Ui) -> WizardAction,
{
    let mut action = WizardAction::NoAction;

    egui::CentralPanel::default().show_inside(ui, |ui| {
        // Header
        let header_height = 64.;
        let header_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), header_height));
        let header_rect = header_space.1;
        let mut painter = ui.painter_at(header_rect);

        painter.text(
            header_rect.center(),
            egui::Align2::CENTER_CENTER,
            title,
            egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
            theme_color().text_on_color_1,
        );

        ui.add_space(4.);
        ui.separator();

        // Content
        let available_height = ui.available_height();
        egui::ScrollArea::vertical()
            .max_height(available_height - 48.) // Subtract footer height
            .show(ui, |ui| {
                let content_height = ui.available_height();
                ui.add_space((content_height - 200.) / 2.0); // Adjust 200 based on estimated content height

                ui.vertical_centered(|ui| {
                    action = content(ui);
                });
            });

        // Footer
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            if ui.large_button(i18n("Back")).clicked() {
                action = WizardAction::Back;
            }
            ui.add_space(4.);
            ui.separator();
            ui.add_space(4.);
        });
    });

    action
}

impl ComponentT for OpenWallet {
  fn render(
    &mut self,
    core: &mut Core,
    ctx: &egui::Context,
    frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    match self.state.clone() {
      State::Select => {
          // let has_stack = core.has_stack();
          let core = Rc::new(RefCell::new(core));
          let mut wallet_descriptor_list = core.borrow_mut().wallet_list.clone();
          let wallet_descriptor_list_is_empty = wallet_descriptor_list.is_empty();

          egui::CentralPanel::default().show_inside(ui, |ui| {
            let header_height = 64.;
            let header_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), header_height));
            let header_rect = header_space.1;
            let mut painter = ui.painter_at(header_rect);

            painter.text(
              header_rect.center(),
              egui::Align2::CENTER_CENTER,
              i18n("Select a Wallet"),
              egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
              theme_color().text_on_color_1,
            );

            ui.add_space(4.);
            ui.separator();
            ui.add_space(4.);

            // Footer
            let footer_height = 48.; // Adjust as needed
            let available_height = ui.available_height();
            let scroll_area_height = available_height - footer_height - 8.; // 8 for spacing

            // Scroll Area with vertical centering
            egui::Frame::none()
              .fill(egui::Color32::TRANSPARENT)
              .inner_margin(0.)
              .outer_margin(0.)
              .show(ui, |ui| {
                egui::ScrollArea::vertical()
                  .max_height(scroll_area_height)
                  .show(ui, |ui| {
                    // Calculate the content height
                    let content_height = if !wallet_descriptor_list_is_empty {
                      (wallet_descriptor_list.len() as f32 * 40.) + 10. // Assuming 40 pixels per button and 10 pixels for spacing
                    } else {
                      0.
                    };

                    let available_space = scroll_area_height.max(content_height);
                    let top_padding = ((available_space - content_height) / 2.0).max(0.0);

                    ui.add_space(top_padding);

                    ui.vertical_centered(|ui| {
                      if !wallet_descriptor_list_is_empty {
                        wallet_descriptor_list.sort();
                        for wallet_descriptor in wallet_descriptor_list.into_iter() {
                          let B = ui.large_button(wallet_descriptor.title.as_deref().unwrap_or_else(||i18n("NO NAME")));
                          if B.clicked() {
                            println!("Opening Wallet {}", wallet_descriptor.title.as_deref().unwrap_or_else(||i18n("NO NAME")));
                          }
                        }
                      }
                    });

                    ui.add_space(top_padding);
                  });
              });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
              let B = ui.large_button(i18n("Add Wallet"));
              if B.clicked() {
                self.args_create.zeroize();
                println!("Creating new wallet");
                self.state = State::KeySelection;
              }
              ui.add_space(4.);
              ui.separator();
              ui.add_space(4.);
            });
        });
      }

      State::KeySelection => {
        let action = render_centered_content(ui, i18n("Choose Wallet Type"), |ui| {
            ui.heading(i18n("Select the number of words for your new seed phrase:"));
            ui.add_space(8.);

            if ui.large_button(i18n("12 Words")).clicked() {
                self.args_create.word_count = WordCount::Words12;
                return WizardAction::Next(State::SetNames);
            }

            ui.add_space(4.);

            if ui.large_button(i18n("24 Words")).clicked() {
                self.args_create.word_count = WordCount::Words24;
                return WizardAction::Next(State::SetNames);
            }

            ui.add_space(24.);

            ui.heading(i18n("OR import an existing wallet:"));

            ui.add_space(8.);
            if ui.large_button(i18n("Import Seed")).clicked() {
                // Handle import seed
            }
            ui.add_space(4.);
            if ui.large_button(i18n("Import File")).clicked() {
                // Handle import file
            }

            WizardAction::NoAction
        });
        match action {
          WizardAction::Back => self.state = State::Select,
          WizardAction::Next(next_state) => self.state = next_state,
          WizardAction::NoAction => {},
        }
      }

      State::SetNames => {
        let action = render_centered_content(ui, i18n("New Wallet & Account Names"), |ui| {
          ui.vertical_centered(|ui| {
            let input_width = 300.0;
            let font_size = 20.0;

            ui.heading(i18n("Wallet Name:"));
            let wallet_name_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.wallet_name)
                .hint_text(i18n("Enter wallet name"))
                .font(egui::FontId::proportional(font_size))
            );
            ui.add_space(10.);

            ui.heading(i18n("Account Name:"));
            let account_name_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.account_name)
                .hint_text(i18n("Enter account name"))
                .font(egui::FontId::proportional(font_size))
            );
            ui.add_space(20.);

            if ui.large_button_enabled(!self.args_create.wallet_name.trim().is_empty(), i18n("Next")).clicked() {
              return WizardAction::Next(State::PhishingHint);
            }
            WizardAction::NoAction
          }).inner
        });

        match action {
          WizardAction::Back => self.state = State::KeySelection,
          WizardAction::Next(next_state) => self.state = next_state,
          WizardAction::NoAction => {},
        }
      }

      State::PhishingHint => {
        let action = render_centered_content(ui, i18n("Phishing Protection"), |ui| {
          ui.vertical_centered(|ui| {
            let input_width = 300.0;
            let font_size = 20.0;

            ui.heading(i18n("Enable Phishing Protection"));
            ui.add_space(8.);
            
            ui.label(i18n("A phishing hint helps you verify that you're using the genuine wallet application."));
            ui.add_space(12.);
            
            ui.label(i18n("Enable phishing protection (recommended)"));
            ui.add(toggle(
              &mut self.args_create.enable_phishing_hint,
            ));
            ui.add_space(12.);

            if self.args_create.enable_phishing_hint {
              ui.heading(i18n("Your Phishing Hint:"));
              let hint_response = ui.add_sized(
                [input_width, 20.0],
                egui::TextEdit::singleline(&mut self.args_create.phishing_hint)
                  .hint_text(i18n("Enter a memorable phrase"))
                  .font(egui::FontId::proportional(font_size))
              );
              ui.add_space(8.);
              ui.label(i18n("This phrase will be shown each time you open your wallet."));
            }

            ui.add_space(20.);

            if ui.large_button_enabled(
              !self.args_create.enable_phishing_hint || 
              !self.args_create.phishing_hint.trim().is_empty(), 
              i18n("Next")
            ).clicked() {
              return WizardAction::Next(State::WalletSecrets);
            }

            WizardAction::NoAction
          }).inner
        });
    
        match action {
          WizardAction::Back => self.state = State::SetNames,
          WizardAction::Next(next_state) => self.state = next_state,
          WizardAction::NoAction => {},
        }
      }

      State::WalletSecrets => {
        let target_opacity = if self.args_create.show_confirmation_popup { 0.66 } else { 0.0 };
        self.args_create.popup_background_opacity = ui.ctx().animate_value_with_time(
          ui.id().with("popup_bg_opacity"),
          target_opacity,
          0.2,
        );    

        let action = render_centered_content(ui, i18n("Set Wallet Passwords"), |ui| {
          ui.vertical_centered(|ui| {
            let input_width = 300.0;
            let font_size = 20.0;

            // Wallet Password Section
            ui.heading(i18n("Wallet Password"));
            ui.add_space(8.);
            
            let password_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.wallet_secret)
                .password(!self.args_create.wallet_secret_show)
                .hint_text(i18n("Enter wallet password"))
                .font(egui::FontId::proportional(font_size))
            );

            let confirm_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.wallet_secret_confirm)
                .password(!self.args_create.wallet_secret_show)
                .hint_text(i18n("Confirm wallet password"))
                .font(egui::FontId::proportional(font_size))
          );

            ui.checkbox(&mut self.args_create.wallet_secret_show, i18n("Show password"));
            
            ui.add_space(20.);

            ui.label(i18n("Enable separate payment password"));
            ui.add(toggle(
              &mut self.args_create.enable_payment_secret,
            ));

            if self.args_create.enable_payment_secret {
              ui.add_space(12.);
              ui.heading(i18n("Payment Password"));
              ui.add_space(8.);

              let payment_response = ui.add_sized(
                [input_width, 20.0],
                egui::TextEdit::singleline(&mut self.args_create.payment_secret)
                  .password(!self.args_create.payment_secret_show)
                  .hint_text(i18n("Enter payment password"))
                  .font(egui::FontId::proportional(font_size))
              );

              let payment_confirm_response = ui.add_sized(
                [input_width, 20.0],
                egui::TextEdit::singleline(&mut self.args_create.payment_secret_confirm)
                  .password(!self.args_create.payment_secret_show)
                  .hint_text(i18n("Confirm payment password"))
                  .font(egui::FontId::proportional(font_size))
              );

              ui.checkbox(&mut self.args_create.payment_secret_show, i18n("Show payment password"));
            }

            ui.add_space(20.);

            let passwords_match = self.args_create.wallet_secret == self.args_create.wallet_secret_confirm &&
              (!self.args_create.enable_payment_secret || 
              self.args_create.payment_secret == self.args_create.payment_secret_confirm);

            let passwords_valid = !self.args_create.wallet_secret.trim().is_empty() &&
              (!self.args_create.enable_payment_secret || 
              !self.args_create.payment_secret.trim().is_empty());

            if ui.large_button_enabled(passwords_match && passwords_valid, i18n("Create Wallet")).clicked() {
              // return WizardAction::Next(State::Create { 
              //   is_pending: false, 
              //   result: Arc::new(Mutex::new(None)) 
              // });
              self.args_create.show_confirmation_popup = true;
            }

            if !passwords_match {
              ui.label(egui::RichText::new(i18n("Passwords do not match"))
                .color(egui::Color32::RED));
            }

            WizardAction::NoAction
          }).inner
        });
    
        match action {
          WizardAction::Back => self.state = State::PhishingHint,
          WizardAction::Next(next_state) => self.state = next_state,
          WizardAction::NoAction => {},
        }

        if self.args_create.popup_background_opacity > 0.0 {
          let screen_rect = ui.ctx().screen_rect();

          let (is_fullscreen, is_maximized) = ctx.input(|i| {
            let viewport = i.viewport();
            (
              viewport.fullscreen.unwrap_or(false),
              viewport.maximized.unwrap_or(false),
            )
          });

          let rounding = if is_fullscreen || is_maximized {
            0.0
          } else {
            crate::frame::WINDOW_ROUNDING
          };

          let mut response = ui.allocate_rect(
            screen_rect, 
            egui::Sense::click_and_drag()
          );
          ui.painter().rect_filled(
            screen_rect,
            rounding,
            egui::Color32::from_black_alpha((self.args_create.popup_background_opacity * 255.0) as u8),
          );
          if response.clicked(){
            self.args_create.show_confirmation_popup = false;
          }
        }

        if self.args_create.show_confirmation_popup {
          egui::Window::new("Confirm Wallet Creation")
              .collapsible(false)
              .resizable(false)
              .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
              .fixed_size([500.0, 0.0])
              .show(ui.ctx(), |ui| {
                  ui.vertical_centered(|ui| {
                      // Header
                      let header_height = 48.;
                      let header_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), header_height));
                      let header_rect = header_space.1;
                      let mut painter = ui.painter_at(header_rect);
      
                      painter.text(
                          header_rect.center(),
                          egui::Align2::CENTER_CENTER,
                          i18n("Wallet Summary"),
                          egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
                          theme_color().text_on_color_1,
                      );
      
                      ui.add_space(8.);
                      ui.separator();
                      ui.add_space(16.);
      
                      // Define consistent sizes
                      let grid_width = 360.0;  // Increased width for larger text
                      let font_size = 18.0;    // Larger font size
      
                      ui.horizontal(|ui| {
                          let indent = (ui.available_width() - grid_width) / 2.0;
                          ui.add_space(indent);
                          egui::Grid::new("confirmation_grid")
                              .spacing([40.0, 8.0])
                              .min_col_width(grid_width / 2.0)  // Ensure consistent column widths
                              .show(ui, |ui| {
                                  // Wallet Name
                                  ui.label(egui::RichText::new(i18n("Wallet Name:"))
                                      .strong()
                                      .size(font_size));
                                  // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                      ui.label(egui::RichText::new(&self.args_create.wallet_name)
                                          .size(font_size));
                                  // });
                                  ui.end_row();
      
                                  // Account Name
                                  ui.label(egui::RichText::new(i18n("Account Name:"))
                                      .strong()
                                      .size(font_size));
                                  // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                      let account_name_display = if self.args_create.account_name.trim().is_empty() {
                                          "Account 0".to_string()
                                      } else {
                                          self.args_create.account_name.clone()
                                      };
                                      ui.label(egui::RichText::new(&account_name_display)
                                          .size(font_size));
                                  // });
                                  ui.end_row();
      
                                  // Word Count
                                  ui.label(egui::RichText::new(i18n("Word Count:"))
                                      .strong()
                                      .size(font_size));
                                  // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                      ui.label(egui::RichText::new(match self.args_create.word_count {
                                          WordCount::Words12 => "12 Words",
                                          WordCount::Words24 => "24 Words",
                                          _ => "Unknown",
                                      }).size(font_size));
                                  // });
                                  ui.end_row();
      
                                  // Phishing Protection
                                  ui.label(egui::RichText::new(i18n("Phishing Protection:"))
                                      .strong()
                                      .size(font_size));
                                  // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                      ui.label(egui::RichText::new(if self.args_create.enable_phishing_hint { 
                                          &self.args_create.phishing_hint 
                                      } else { 
                                          "Disabled"
                                      }).size(font_size));
                                  // });
                                  ui.end_row();
                              });
                      });
      
                      ui.add_space(12.);
                      ui.checkbox(
                          &mut self.args_create.show_secrets_in_popup, 
                          egui::RichText::new(i18n("Show Passwords")).size(font_size)
                      );
                      ui.add_space(8.);
      
                      ui.horizontal(|ui| {
                          let indent = (ui.available_width() - grid_width) / 2.0;
                          ui.add_space(indent);
                          egui::Grid::new("passwords_grid")
                              .spacing([40.0, 8.0])
                              .min_col_width(grid_width / 2.0)  // Match the width of the first grid
                              .show(ui, |ui| {
                                  // Wallet Password
                                  ui.label(egui::RichText::new(i18n("Wallet Password:"))
                                      .strong()
                                      .size(font_size));
                                  // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                      ui.label(egui::RichText::new(if self.args_create.show_secrets_in_popup { 
                                          self.args_create.wallet_secret.clone()
                                      } else { 
                                          "*".repeat(self.args_create.wallet_secret.len()) 
                                      }).size(font_size));
                                  // });
                                  ui.end_row();
      
                                  // Payment Password
                                  if self.args_create.enable_payment_secret {
                                      ui.label(egui::RichText::new(i18n("Payment Password:"))
                                          .strong()
                                          .size(font_size));
                                      // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                          ui.label(egui::RichText::new(if self.args_create.show_secrets_in_popup { 
                                              self.args_create.payment_secret.clone() 
                                          } else { 
                                              "*".repeat(self.args_create.payment_secret.len()) 
                                          }).size(font_size));
                                      // });
                                      ui.end_row();
                                  }
                              });
                      });
      
                      ui.add_space(24.);
      
                      // Buttons
                      ui.horizontal(|ui| {
                          let button_width = 175.0;
                          ui.add_space((ui.available_width() - (button_width * 2.0 + 8.0)) / 2.0);
                          
                          if ui.add_sized([button_width, 40.0], egui::Button::new(
                              egui::RichText::new(i18n("Cancel")).size(font_size)
                          )).clicked() {
                              self.args_create.show_secrets_in_popup = false;
                              self.args_create.show_confirmation_popup = false;
                          }
                          ui.add_space(8.);
                          if ui.add_sized([button_width, 40.0], egui::Button::new(
                              egui::RichText::new(i18n("Confirm and Create")).size(font_size)
                          )).clicked() {
                              self.args_create.show_secrets_in_popup = false;
                              self.args_create.show_confirmation_popup = false;
                              self.state = State::Create { 
                                  is_pending: false, 
                                  result: Arc::new(Mutex::new(None)) 
                              };
                          }
                      });
                  });
              });
      }
      }
      
      State::Create { is_pending, result } => {
        egui::CentralPanel::default().show_inside(ui, |ui| {
          let header_height = 64.;
          let header_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), header_height));
          let header_rect = header_space.1;
          let mut painter = ui.painter_at(header_rect);

          painter.text(
            header_rect.center(),
            egui::Align2::CENTER_CENTER,
            i18n("Wallet Creation"),
            egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
            theme_color().text_on_color_1,
          );

          ui.add_space(4.);
          ui.separator();
          ui.add_space(4.);

          // if !*is_pending {
          //   let args_create = self.args_create.clone();
          //   let wallet = self.manager.wallet().clone();
          //   let result_clone = result.clone();
          // }
        });
      }

      _ => {}
    }
  }
}