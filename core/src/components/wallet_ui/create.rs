use crate::imports::*;
use waglayla_wallet_core::{wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, encryption::EncryptionKind, api::{AccountsDiscoveryRequest, AccountsDiscoveryKind}};
use waglayla_bip32::{WordCount, Mnemonic, Language};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;
use core::cmp::max;

use crate::components::wallet_ui::BOTTOM_SPACE;

use rand::seq::SliceRandom; // For shuffling
use rand::thread_rng;       // Random number generator

#[derive(Clone, Default)]
pub enum State {
  #[default]
  Select,
  KeySelection,
  SetNames,
  PhishingHint,
  WalletSecrets,
  Create { is_pending: Arc<Mutex<bool>>, result: Arc<Mutex<Option<Result<(String, AccountDescriptor)>>>> },
  PresentSeed(String),
  Unlock { wallet_descriptor : WalletDescriptor, error : Option<Arc<Error>>},
  Unlocking { wallet_descriptor : WalletDescriptor },
}

enum WizardAction {
  Back,
  Next(State),
  NoAction,
}

impl WizardActionTrait for WizardAction {
  fn is_no_action(&self) -> bool {
    matches!(self, WizardAction::NoAction)
  }

  fn is_back(&self) -> bool {
    matches!(self, WizardAction::Back)
  }

  fn from_back() -> Self {
    WizardAction::Back
  }
}

#[derive(Clone, Default)]
enum SeedState {
  #[default]
  Display,
  Confirm1(i32),
  Confirm2(i32),
  Confirm3(i32)
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
  mnemonic_presenter_context : MnemonicPresenterContext,
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
    self.mnemonic_presenter_context.zeroize();

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

pub struct CreateWallet {
  #[allow(dead_code)]
  manager: DX_Manager,
  wallet_secret: String,
  args_create: CreationArgs,
  pub state: State,
  pub seed_state: SeedState,
  pub seed_shuffle: Vec<usize>,

  pub import_key: bool,
  pub import_file: bool,

  confirm_1: String,
  confirm_2: String,
  confirm_3: String,

  pub message: Option<String>,
}

impl CreateWallet {
  pub fn new(manager: DX_Manager) -> Self {
    Self {
      manager,
      wallet_secret: String::new(),
      args_create: Default::default(),
      state: State::Select,
      seed_state: Default::default(),
      seed_shuffle: Vec::new(),

      import_key: false,
      import_file: false,

      confirm_1: String::new(),
      confirm_2: String::new(),
      confirm_3: String::new(),
      message: None,
    }
  }

  fn shuffle_seed(&mut self, count: usize) {
    let mut rng = thread_rng();
    self.seed_shuffle = (0..count).collect::<Vec<_>>();
    self.seed_shuffle.shuffle(&mut rng);
  }

  pub fn open(&mut self, wallet_descriptor: WalletDescriptor) {
    self.state = State::Unlock { wallet_descriptor, error : None};
  }
}

fn get_shuffled_vector(count: usize) -> Vec<usize> {
  let mut rng = thread_rng(); // Create a random number generator
  let mut numbers: Vec<usize> = (0..count).collect(); // Create a vector [0, 1, ..., count-1]
  numbers.shuffle(&mut rng); // Shuffle the vector in place
  numbers
}

impl ComponentT for CreateWallet {
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

          egui::CentralPanel::default()
          .frame(create_custom_frame(ctx))
          .show_inside(ui, |ui| {
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
                  .max_height(scroll_area_height - 56.)
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
                          let B = ui.dx_button_sized(
                            wallet_descriptor.title.as_deref().unwrap_or_else(||i18n("NO NAME")), 
                            32.0,
                            -14.0, 
                            Default::default(), 
                            vec2(220.0, 48.0)
                          );
                          ui.add_space(6.0);

                          if B.clicked() {
                            // println!("Opening Wallet {}", wallet_descriptor.title.as_deref().unwrap_or_else(||i18n("NO NAME")));
                            core.borrow_mut().get_mut::<components::wallet_ui::OpenWallet>()
                              .open(wallet_descriptor.clone());
                            core.borrow_mut().set_active_component::<crate::components::wallet_ui::OpenWallet>();
                          }
                        }
                      }
                    });

                    ui.add_space(top_padding);
                  });
              });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
              ui.add_space(BOTTOM_SPACE);
              let B = ui.dx_large_button(i18n("Add Wallet"));
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
        let action = render_centered_content(ctx, ui, i18n("Choose Wallet Type"), |ui| {
            ui.heading(i18n("Select the number of words for your new seed phrase:"));
            ui.add_space(8.);

            if ui.dx_button_sized(
              i18n("12 Words"), 
              26.0,
              -13.0, 
              Default::default(), 
              vec2(180.0, 32.0)
            ).clicked() {
              self.args_create.word_count = WordCount::Words12;
              return WizardAction::Next(State::SetNames);
            }

            ui.add_space(6.);

            if ui.dx_button_sized(
              i18n("24 Words"), 
              26.0,
              -13.0, 
              Default::default(), 
              vec2(180.0, 32.0)
            ).clicked() {
              self.args_create.word_count = WordCount::Words24;
              return WizardAction::Next(State::SetNames);
            }

            ui.add_space(20.);

            ui.heading(i18n("OR import an existing wallet:"));

            ui.add_space(8.);
            if ui.dx_button_sized(
              i18n("Import Key"), 
              26.0,
              -13.0, 
              Default::default(), 
              vec2(180.0, 32.0)
            ).clicked() {
                // Handle import seed
            }

            ui.add_space(6.);
            if ui.dx_button_sized(
              i18n("Import File"), 
              26.0,
              -13.0, 
              Default::default(), 
              vec2(180.0, 32.0)
            ).clicked() {
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
        let action = render_centered_content(ctx, ui, i18n("Wallet Creation"), |ui| {
          ui.vertical_centered(|ui| {
            let input_width = 300.0;
            let font_size = 20.0;

            ui.heading(i18n("Wallet:"));
            let wallet_name_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.wallet_name)
                .hint_text(i18n("Enter wallet name"))
                .font(egui::FontId::proportional(font_size))
            );
            ui.add_space(10.);

            ui.heading(i18n("Default Account (Optional):"));
            let account_name_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.account_name)
                .hint_text(i18n("Enter account name"))
                .font(egui::FontId::proportional(font_size))
            );
            ui.add_space(20.);

            if ui.dx_large_button_enabled(!self.args_create.wallet_name.trim().is_empty(), i18n("Next")).clicked() {
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
        let action = render_centered_content(ctx, ui, i18n("Phishing Protection"), |ui| {
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

            if ui.dx_large_button_enabled(
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

        let action = render_centered_content(ctx, ui, i18n("Passwords"), |ui| {
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

            if ui.dx_large_button_enabled(passwords_match && passwords_valid, i18n("Create Wallet")).clicked() {
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

            ui.label("");

            WizardAction::NoAction
          }).inner
        });
    
        match action {
          WizardAction::Back => {
            self.args_create.popup_background_opacity = 0.0;
            self.state = State::PhishingHint;
          },
          WizardAction::Next(next_state) => {
            self.args_create.popup_background_opacity = 0.0;
            self.state = next_state;
          },
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
          let mut F = create_custom_popup(ctx);
          F.rounding = 10.0.into();
          egui::Window::new("")
              .collapsible(false)
              .resizable(false)
              .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
              .fixed_size([500.0, 0.0])
              .frame(F)
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
                                  ui.label(egui::RichText::new(i18n("Wallet:"))
                                      .strong()
                                      .size(font_size));
                                  // ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                      ui.label(egui::RichText::new(&self.args_create.wallet_name)
                                          .size(font_size));
                                  // });
                                  ui.end_row();
      
                                  // Account Name
                                  ui.label(egui::RichText::new(i18n("Default Account:"))
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
                                  ui.label(egui::RichText::new(i18n("Seed Length:"))
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

                          let mut resp = ui.dx_button_sized(
                            i18n("Cancel"), 
                            24.0, 
                            -12.0, 
                            Default::default(), 
                            vec2(button_width, 40.0),
                          );

                          resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                          if resp.clicked() {
                              self.args_create.show_secrets_in_popup = false;
                              self.args_create.show_confirmation_popup = false;
                          }
                          ui.add_space(8.);

                          let mut resp = ui.dx_button_sized(
                            i18n("Confirm and Create"), 
                            24.0, 
                            -12.0, 
                            Default::default(), 
                            vec2(button_width, 40.0),
                          );

                          if resp.clicked() {
                              self.args_create.show_secrets_in_popup = false;
                              self.args_create.show_confirmation_popup = false;
                              self.state = State::Create { 
                                  is_pending: Arc::new(Mutex::new(false)), 
                                  result: Arc::new(Mutex::new(None)) 
                              };
                          }
                      });

                      ui.add_space(16.);
                  });
              });
      }
      }
      
      State::Create { is_pending, result } => {
        let mut is_pending = is_pending.clone();
        // Display the "Creating Wallet" UI
        render_centered_content_noback(ctx, ui, i18n("Creating Wallet"), |ui| {
            ui.vertical_centered(|ui| {
              ui.label(i18n("Please wait..."));
              ui.add_space(8.0);
              ui.add(DX_Spinner::new()
                .size(125.0)
                .color(theme_color().strong_color)
                .stroke_width(12.0)
              );
            });
            WizardAction::NoAction
          });
    
        // If the task is not yet running, start the asynchronous process
        if !*is_pending.lock().unwrap() {
          *is_pending.lock().unwrap() = true;
      
          let args_create = self.args_create.clone();
          let wallet = self.manager.wallet().clone();
          let mut result = result.clone();
      
          tokio::spawn(async move {
            let res = async {
              if args_create.enable_payment_secret && args_create.payment_secret.is_empty() {
                return Err(Error::custom(i18n("Payment secret is empty")));
              }
  
              if args_create.enable_phishing_hint && args_create.phishing_hint.is_empty() {
                return Err(Error::custom(i18n("Phishing hint is empty")));
              }
  
              let wallet_secret = Secret::from(args_create.wallet_secret);
              let payment_secret = args_create
                .enable_payment_secret
                .then_some(Secret::from(args_create.payment_secret));
  
              {
                // Ensure exclusive access to the wallet
                wallet.clone().batch().await?;

                let wallet_args = WalletCreateArgs::new(
                  args_create.wallet_name.is_not_empty().then_some(args_create.wallet_name),
                  args_create.wallet_filename.is_not_empty().then_some(args_create.wallet_filename),
                  EncryptionKind::XChaCha20Poly1305,
                  args_create.enable_phishing_hint.then_some(args_create.phishing_hint.into()),
                  false,
                );

                wallet.clone().wallet_create(wallet_secret.clone(), wallet_args).await?;

                let mnemonic = Mnemonic::random(args_create.word_count, Language::default())?;
                let mnemonic_phrase_string = mnemonic.phrase_string();
                let prv_key_data_args = PrvKeyDataCreateArgs::new(
                  None,
                  payment_secret.clone(),
                  Secret::from(mnemonic_phrase_string.clone()),
                );

                let prv_key_data_id = wallet.clone()
                  .prv_key_data_create(wallet_secret.clone(), prv_key_data_args)
                  .await?;

                let account_name = if args_create.account_name.trim().is_empty() {
                  "Account 0".to_string()
                } else {
                  args_create.account_name.clone()
                };

                let account_create_args = AccountCreateArgs::new_bip32(
                  prv_key_data_id,
                  payment_secret.clone(),
                  Some(account_name),
                  None,
                );

                let account_descriptor = wallet.clone()
                  .accounts_create(wallet_secret.clone(), account_create_args)
                  .await?;

                wallet.clone().flush(wallet_secret).await?;
                
                Ok((mnemonic_phrase_string, account_descriptor))
              }
            }
            .await;

            *result.lock().unwrap() = Some(res);
          });
      }
    
        // Check if the asynchronous task has completed
        if let Some(res) = result.lock().unwrap().take() {
            match res {
                Ok((mnemonic, account_descriptor)) => {
                    self.args_create.zeroize();
                    core.handle_account_creation(vec![account_descriptor]);
                    self.seed_state = SeedState::Display;
                    self.state = State::PresentSeed(mnemonic);
                }
                Err(err) => {
                    log_error!("{} {}", i18n("Wallet creation error:"), err);
                    self.state = State::Select;
                }
            }
        }
    }

    State::PresentSeed(mut mnemonic) => {
        let mut finish = false;
        let mut F = create_custom_popup(ctx);
        F.rounding = 10.0.into();
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
          egui::Color32::from_black_alpha((0.66 * 255.0) as u8),
        );
        if response.clicked(){}

        let h_scale = max(450, (ui.available_width() / 1.5) as i32);
        let v_scale = max(450, (ui.available_height() / 1.33) as i32);

        egui::Window::new(i18n("Mnemonic Seed Phrase"))
          .collapsible(false)
          .resizable(false)
          .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
          .max_size([h_scale as f32, v_scale as f32])
          .frame(F)
          .show(ui.ctx(), |ui| {
            let available_height = ui.available_height();

            match self.seed_state {
              SeedState::Display => {
                egui::ScrollArea::vertical()
                  .max_height(available_height - 56.) 
                  .show(ui, |ui| {
                    // render_centered_content_noback(ctx, ui, i18n("Mnemonic Seed Phrase"), |ui| {
                    let mut mnemonic_presenter = MnemonicPresenter::new(mnemonic.as_str(), &mut self.args_create.mnemonic_presenter_context);
    
                    ui.vertical_centered(|ui| {
                      ui.label(RichText::new(i18n(mnemonic_presenter.notice())).size(14.));
                      ui.label("");
                      ui.label(RichText::new(i18n(mnemonic_presenter.warning())).size(14.));
                    });
    
                    ui.label("");
                    mnemonic_presenter.render(ui, Some(i18n("Your default wallet private key mnemonic is:")));
                    ui.label("");
                });
    
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                  ui.add_space(BOTTOM_SPACE);
                  if ui.dx_large_button(i18n("Continue")).clicked() {
                    finish = true;
                  }
                  ui.add_space(4.);
                  ui.separator();
                  ui.add_space(4.);
                });
  
                if finish {
                  // this.state = State::ConfirmMnemonic(mnemonic);
                  mnemonic.zeroize();
                  let count = match self.args_create.word_count {
                    WordCount::Words12 => 12,
                    WordCount::Words24 => 24
                  };

                  self.shuffle_seed(count);
                  self.seed_state = SeedState::Confirm1(self.seed_shuffle[0] as i32);
                } 
              }

              SeedState::Confirm1(index) | SeedState::Confirm2(index) | SeedState::Confirm3(index) => {
                let word_index = index as usize;
                let words = mnemonic.split(' ').collect::<Vec<_>>();
                let correct_word = words.get(word_index).unwrap_or(&"");

                let mut input_word = match self.seed_state.clone() {
                  SeedState::Confirm1(index) => {
                    &mut self.confirm_1
                  }
                  SeedState::Confirm2(index) => {
                    &mut self.confirm_2
                  }
                  SeedState::Confirm3(index) => {
                    &mut self.confirm_3
                  }
                  _ => {&mut self.confirm_1}
                };
                let mut is_correct = false;

                render_centered_content_noback(ctx, ui, i18n("Seed Confirmation"), |ui| {
                  ui.vertical_centered(|ui| {
                    ui.heading(format!(
                      "{}",
                      i18n(&format!(
                        "Please enter word {} from your seed phrase:",
                        word_index + 1
                      ))
                    ));
                    ui.add_space(8.);

                    ui.add_sized(
                      [300.0, 20.0],
                      egui::TextEdit::singleline(input_word)
                        .hint_text(i18n("Type your word here"))
                        .font(egui::FontId::proportional(18.0)),
                    );

                    is_correct = input_word.trim() == *correct_word;

                    if !is_correct {
                      ui.label(RichText::new(i18n("Word does not match")).color(egui::Color32::RED));
                    }
                  });
                  WizardAction::NoAction
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                  ui.add_space(BOTTOM_SPACE);
              
                  ui.horizontal(|ui| {
                    let button_width = 175.0;
                    let button_height = 40.0;
            
                    // Calculate the left and right spacing to center the buttons
                    ui.add_space((ui.available_width() - (button_width * 2.0 + 8.0)) / 2.0);
            
                    // Back button
                    let mut back_button = ui.add_sized([button_width, button_height], egui::Button::new(
                        egui::RichText::new(i18n("Back")).size(18.0)
                    ));
                    back_button = back_button.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if back_button.clicked() {
                      match self.seed_state {
                        SeedState::Confirm3(_) => {
                          self.seed_state = SeedState::Confirm2(self.seed_shuffle[1] as i32);
                        }
                        SeedState::Confirm2(_) => {
                          self.seed_state = SeedState::Confirm1(self.seed_shuffle[0] as i32);
                        }
                        SeedState::Confirm1(_) => {
                          self.seed_state = SeedState::Display;
                        }
                        _ => {}
                      }
                    }
            
                    ui.add_space(8.);
            
                    // Continue button
                    let mut continue_button = ui.add_enabled(
                      is_correct,
                      egui::Button::new(egui::RichText::new(i18n("Continue")).size(18.0))
                        .min_size(egui::Vec2::new(button_width, button_height))
                    );

                    continue_button = continue_button.on_hover_cursor(egui::CursorIcon::PointingHand);

                    if continue_button.clicked() {
                      match self.seed_state {
                        SeedState::Confirm1(_) => {
                          self.seed_state = SeedState::Confirm2(self.seed_shuffle[1] as i32);
                        }
                        SeedState::Confirm2(_) => {
                          self.seed_state = SeedState::Confirm3(self.seed_shuffle[2] as i32);
                        }
                        SeedState::Confirm3(_) => {
                          finish = true;
                        }
                        _ => {}
                      }
                    }
                  });
              
                  ui.add_space(4.0);
                  ui.separator();
                  ui.add_space(4.0);
                });

                if finish {
                  mnemonic.zeroize();
                  self.state = State::Select; // Proceed to final confirmation or success state
                }
              }

                _ => {}
              }
            });
          }

      _ => {}
    }
  }
}