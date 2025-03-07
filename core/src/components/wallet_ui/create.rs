use crate::imports::*;
use waglayla_wallet_core::{wallet::{AccountCreateArgs, PrvKeyDataCreateArgs, WalletCreateArgs}, encryption::EncryptionKind};
use waglayla_bip32::{WordCount, Mnemonic, Language};
use std::sync::{Arc, Mutex};
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
  EnterKey(String),
  ImportKey { is_pending: Arc<Mutex<bool>>, result: Arc<Mutex<Option<Result<AccountDescriptor>>>> },
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

define_indexed_enum!(
  Focus,
  WalletName,
  AccountName,
  PhishingHint,
  WalletSecret,
  WalletConfirm,
  PaymentSecret,
  PaymentConfirm,
  SeedPhrase,
  WordCheck
);

#[derive(Clone)]
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

  import_key_input: String,

  confirm_1: String,
  confirm_2: String,
  confirm_3: String,

  pub message: Option<String>,
  focus_context: FocusContext,
}

impl_has_focus_context!(CreateWallet);

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

      import_key_input: String::new(),

      confirm_1: String::new(),
      confirm_2: String::new(),
      confirm_3: String::new(),
      message: None,

      focus_context: FocusContext { focus: FOCUS_NONE },
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

  // pub fn render_back_button(
  //   core: &mut Core,
  //   &mut self, 
  //   ui: &mut egui::Ui, 
  //   back_to: ComponentT
  // ) {
  //   let available_rect = ui.available_rect();
  //   let back_pos = egui::Pos2 {
  //     x: available_rect.min.x + 16.0,
  //     y: available_rect.min.y + 16.0,
  //   };

  //   let back_size = 48.0;
  //   let back_icon = egui_phosphor::fill::ARROW_U_UP_LEFT;

  //   let galley = ui.fonts(|f| {
  //     f.layout_no_wrap(
  //       back_icon,
  //       egui::FontId::new(back_size, egui::FontFamily::Name("phosphor-fill".into())),
  //       theme_color().strong_color,
  //     )
  //   });

  //   let back_rect = egui::Rect::from_min_max(
  //     egui::Pos2 {
  //       x: back_pos.x,
  //       y: back_pos.y,
  //     },
  //     egui::Pos2 {
  //       x: back_pos.x + galley.rect.width(),
  //       y: back_pos.y + galley.rect.height(),
  //     },
  //   );

  //   let mut response = ui.interact(
  //     back_rect, 
  //     egui::Id::new("back_btn_area"), 
  //     egui::Sense::click() | egui::Sense::hover(),
  //   );

  //   let back_color = if response.hovered() {
  //     theme_color().strong_color
  //   } else {
  //     theme_color().text_off_color_1
  //   };

  //   let color = ui.ctx().animate_color_with_time(
  //     response.id.with("back_btn_color"),
  //     back_color,
  //     0.125
  //   );  

  //   ui.painter().text(
  //     back_pos,
  //     egui::Align2::LEFT_TOP,
  //     back_icon,
  //     egui::FontId::new(16.0, get_font_family("DINishCondensed", false, false)),
  //     color,
  //   );

  //   if response.clicked() {
      
  //   }

  //   response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
  // }
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
            let painter = ui.painter_at(header_rect);

            let has_account = core.borrow().current_account.is_some();
            if has_account {
              
            }

            painter.text(
              header_rect.center() + vec2(0.0, 8.0),
              egui::Align2::CENTER_CENTER,
              i18n("Select a Wallet"),
              egui::FontId::new(header_height * 0.75, get_font_family("DINish", false, false)),
              theme_color().text_on_color_1,
            );

            ui.add_space(4.);
            ui.separator();
            ui.add_space(4.);

            // Footer
            let footer_height = 48.;
            let available_height = ui.available_height();
            let scroll_area_height = available_height - footer_height - 8.;

            // Scroll Area with vertical centering
            egui::Frame::none()
              .fill(egui::Color32::TRANSPARENT)
              .inner_margin(0.)
              .outer_margin(0.)
              .show(ui, |ui| {
                egui::ScrollArea::vertical()
                  .max_height(scroll_area_height - 4.)
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
                            Default::default(), 
                            vec2(220.0, 48.0)
                          );
                          ui.add_space(6.0);

                          if B.clicked() {
                            log_info!("Opening Wallet {}", wallet_descriptor.title.as_deref().unwrap_or_else(||i18n("NO NAME")));
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
        let action = render_centered_content(ctx, ui, i18n("Setup Wallet Seed"), 230.0, |ui| {
          ui.heading(i18n("Select your seed word count:"));
          ui.add_space(8.);

          if ui.dx_button_sized(
            i18n("12 Words"), 
            26.0,
            Default::default(), 
            vec2(180.0, 32.0)
          ).clicked() {
            self.import_key = false;
            self.import_file = false;
            self.args_create.word_count = WordCount::Words12;
            return WizardAction::Next(State::SetNames);
          }

          ui.add_space(6.);

          if ui.dx_button_sized(
            i18n("24 Words"), 
            26.0,
            Default::default(), 
            vec2(180.0, 32.0)
          ).clicked() {
            self.import_key = false;
            self.import_file = false;
            self.args_create.word_count = WordCount::Words24;
            return WizardAction::Next(State::SetNames);
          }

          ui.add_space(20.);

          ui.heading(i18n("OR import an existing wallet:"));

          ui.add_space(8.);
          if ui.dx_button_sized(
            i18n("Import Seed"), 
            26.0,
            Default::default(), 
            vec2(180.0, 32.0)
          ).clicked() {
            self.import_key = true;
            self.import_file = false;
            return WizardAction::Next(State::SetNames);
          }

          // TODO if needed in the future
          // ui.add_space(6.);
          // if ui.dx_button_sized(
          //   i18n("Import File"), 
          //   26.0,
          //   -13.0, 
          //   Default::default(), 
          //   vec2(180.0, 32.0)
          // ).clicked() {
          //   self.import_key = false;
          //   self.import_file = true;
          //     // Handle import file
          // }

          WizardAction::NoAction
        });
        match action {
          WizardAction::Back => self.state = State::Select,
          WizardAction::Next(next_state) => {
            match next_state {
              State::SetNames => {
                self.assign_focus(Focus::WalletName);
              }

              _ => {}
            }
            self.state = next_state;
          },
          WizardAction::NoAction => {},
        }
      }

      State::SetNames => {
        let header = match self.import_key {
          true => i18n("Import Wallet"),
          false => i18n("Wallet Creation"),
        };

        let enabled = !self.args_create.wallet_name.trim().is_empty()
            && !self.args_create.account_name.trim().is_empty();

        let action = render_centered_content(ctx, ui, header, 200.0, |ui| {
          ui.vertical_centered(|ui| {
            let input_width = 300.0;
            let font_size = 20.0;

            ui.heading(i18n("Wallet Name:"));
            let wallet_name_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.wallet_name)
                .hint_text(i18n("Type wallet name here"))
                .font(egui::FontId::proportional(font_size))
            );
            self.next_focus(ui, Focus::WalletName, wallet_name_response.clone());
            if wallet_name_response.lost_focus() && handle_enter_key(ui) {
              self.assign_focus(Focus::AccountName);
            }
            ui.add_space(10.);

            ui.heading(i18n("Account Name:"));
            let account_name_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.account_name)
                .hint_text(i18n("Type account name here"))
                .font(egui::FontId::proportional(font_size))
            );
            self.next_focus(ui, Focus::AccountName, account_name_response.clone());
            if account_name_response.lost_focus() && handle_enter_key(ui) {
              if enabled {
                return WizardAction::Next(State::PhishingHint);
              }
            }

            ui.add_space(20.);

            if ui.dx_large_button_enabled(enabled, i18n("Next")).clicked() {
              return WizardAction::Next(State::PhishingHint);
            }
            WizardAction::NoAction
          }).inner
        });

        match action {
          WizardAction::Back => self.state = State::KeySelection,
          WizardAction::Next(next_state) => {
            match next_state {
              State::PhishingHint => {
                self.assign_focus(Focus::PhishingHint);
              }
              _ => {}
            }
            self.state = next_state;
          },
          WizardAction::NoAction => {},
        }
      }

      State::PhishingHint => {
        let action = render_centered_content(ctx, ui, i18n("Phishing Protection"), 200.0, |ui| {
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
              self.next_focus(ui, Focus::PhishingHint, hint_response.clone());
              if hint_response.lost_focus() && handle_enter_key(ui) {
                if !self.args_create.enable_phishing_hint || 
                  !self.args_create.phishing_hint.trim().is_empty() 
                {
                  return WizardAction::Next(State::WalletSecrets);
                }
              }
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
          WizardAction::Back => {
            self.assign_focus(Focus::WalletName);
            self.state = State::SetNames;
          },
          WizardAction::Next(next_state) => {
            match next_state {
              State::WalletSecrets => {
                self.assign_focus(Focus::WalletSecret);
              }
              _ => {}
            }
            self.state = next_state;
          },
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

        let passwords_match = self.args_create.wallet_secret == self.args_create.wallet_secret_confirm &&
          (!self.args_create.enable_payment_secret || 
          self.args_create.payment_secret == self.args_create.payment_secret_confirm);

        let passwords_valid = !self.args_create.wallet_secret.trim().is_empty() &&
          (!self.args_create.enable_payment_secret || 
          !self.args_create.payment_secret.trim().is_empty());

        let action = render_centered_content(ctx, ui, i18n("Passwords"), 200.0, |ui| {
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
                .hint_text(i18n("Type password here"))
                .font(egui::FontId::proportional(font_size))
            );
            self.next_focus(ui, Focus::WalletSecret, password_response.clone());

            let confirm_response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.args_create.wallet_secret_confirm)
                .password(!self.args_create.wallet_secret_show)
                .hint_text(i18n("Confirm password here"))
                .font(egui::FontId::proportional(font_size))
            );
            self.next_focus(ui, Focus::WalletConfirm, confirm_response.clone());
            if confirm_response.lost_focus() && handle_enter_key(ui) {
              if self.args_create.enable_payment_secret {
                self.assign_focus(Focus::PaymentSecret);
              } else {
                if passwords_match && passwords_valid {
                  if self.import_key {
                    return WizardAction::Next(State::EnterKey(String::new()));
                  } else {
                    self.args_create.show_confirmation_popup = true;
                  }
                } else {
                  self.assign_focus(Focus::WalletConfirm);
                }
              }
            }

            if password_response.lost_focus() && handle_enter_key(ui) {
              self.assign_focus(Focus::WalletConfirm);
            }

            ui.checkbox(&mut self.args_create.wallet_secret_show, i18n("Show password"));
            
            ui.add_space(20.);

            ui.heading(i18n("Enable separate payment password"));
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
              self.next_focus(ui, Focus::PaymentSecret, payment_response.clone());

              let payment_confirm_response = ui.add_sized(
                [input_width, 20.0],
                egui::TextEdit::singleline(&mut self.args_create.payment_secret_confirm)
                  .password(!self.args_create.payment_secret_show)
                  .hint_text(i18n("Confirm payment password"))
                  .font(egui::FontId::proportional(font_size))
              );
              self.next_focus(ui, Focus::PaymentConfirm, payment_confirm_response.clone());

              if payment_response.lost_focus() && handle_enter_key(ui) {
                self.assign_focus(Focus::PaymentConfirm);
              }
              if payment_confirm_response.lost_focus() && handle_enter_key(ui) {
                if passwords_match && passwords_valid {
                  if self.import_key {
                    return WizardAction::Next(State::EnterKey(String::new()));
                  } else {
                    self.args_create.show_confirmation_popup = true;
                  }
                } else {
                  self.assign_focus(Focus::PaymentConfirm);
                }
              }

              ui.checkbox(&mut self.args_create.payment_secret_show, i18n("Show payment password"));
            }

            ui.add_space(20.);

            let label = match self.import_key {
              true => i18n("Import Wallet"),
              false => i18n("Create Wallet"),
            };
            if ui.dx_large_button_enabled(passwords_match && passwords_valid, label).clicked() {
              if self.import_key {
                return WizardAction::Next(State::EnterKey(String::new()));
              } else {
                self.args_create.show_confirmation_popup = true;
              }
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
            self.assign_focus(Focus::PhishingHint);
          },
          WizardAction::Next(next_state) => {
            match next_state {
              State::EnterKey (ref string) => {
                self.assign_focus(Focus::SeedPhrase);
              }
              _ => {}
            }
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
            crate::gui::frame::WINDOW_ROUNDING
          };

          let response = ui.allocate_rect(
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
                      let painter = ui.painter_at(header_rect);
      
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
                                      let account_name_display = self.args_create.account_name.to_owned();
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
                            Default::default(), 
                            vec2(button_width, 40.0),
                          );

                          resp = resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                          if resp.clicked() {
                              self.args_create.show_secrets_in_popup = false;
                              self.args_create.show_confirmation_popup = false;
                          }
                          ui.add_space(8.);

                          let resp = ui.dx_button_sized(
                            i18n("Confirm and Create"), 
                            24.0, 
                            Default::default(), 
                            vec2(button_width, 40.0),
                          );

                          if resp.clicked() {
                              self.args_create.show_secrets_in_popup = false;
                              self.args_create.show_confirmation_popup = false;
                              self.assign_focus(Focus::WordCheck);
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
        let is_pending = is_pending.clone();
        // Display the "Creating Wallet" UI
        render_centered_content_noback(ctx, ui, i18n("Creating Wallet"), 200.0, |ui| {
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
          let result = result.clone();
      
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

                let account_name = args_create.account_name.to_owned();

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
          crate::gui::frame::WINDOW_ROUNDING
        };

        let response = ui.allocate_rect(
          screen_rect, 
          egui::Sense::click_and_drag()
        );
        ui.painter().rect_filled(
          screen_rect,
          rounding,
          egui::Color32::from_black_alpha((0.66 * 255.0) as u8),
        );
        if response.clicked(){}

        let h_scale = max(500, (ui.available_width() / 1.5) as i32);
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
                  .max_height(available_height - 60.) 
                  .show(ui, |ui| {
                    // render_centered_content_noback(ctx, ui, i18n("Mnemonic Seed Phrase"), |ui| {
                    let mut mnemonic_presenter = MnemonicPresenter::new(mnemonic.as_str(), &mut self.args_create.mnemonic_presenter_context);
    
                    ui.vertical_centered(|ui| {
                      ui.label(RichText::new(i18n(mnemonic_presenter.notice())).size(14.));
                      ui.label("");
                      ui.label(RichText::new(i18n(mnemonic_presenter.warning())).size(14.));
                    });
    
                    ui.label("");
                    mnemonic_presenter.render(core, ui, Some(i18n("Your base wallets mnemonic seed is:")));
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

                let mut continued = false;

                let mut is_correct = false;

                let title = match self.seed_state.to_owned() {
                  SeedState::Confirm1(index) => {
                    i18n("Confirmation 1/3")
                  }
                  SeedState::Confirm2(index) => {
                    i18n("Confirmation 2/3")
                  }
                  SeedState::Confirm3(index) => {
                    i18n("Confirmation 3/3")
                  },
                  _ => "",
                };

                render_centered_content_noback(ctx, ui, title, 200.0, |ui| {
                  ui.vertical_centered(|ui| {
                    let input_word = match self.seed_state.to_owned() {
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

                    ui.heading(format!(
                      "{}",
                      i18n(&format!(
                        "Please enter word {} from your seed phrase:",
                        word_index + 1
                      ))
                    ));
                    ui.add_space(8.);

                    let word_response = ui.add_sized(
                      [300.0, 20.0],
                      egui::TextEdit::singleline(input_word)
                        .hint_text(i18n("Type your word here"))
                        .font(egui::FontId::proportional(18.0)),
                    );

                    is_correct = input_word.trim() == *correct_word;

                    if !is_correct {
                      ui.label(RichText::new(i18n("Word does not match")).color(egui::Color32::RED));
                    } else {
                      if word_response.lost_focus() && handle_enter_key(ui) {
                        continued = true;
                      }
                    }

                    self.next_focus(ui, Focus::WordCheck, word_response.to_owned());
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

                    if continued || continue_button.clicked() {
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
                  core.set_active_component::<components::wallet_ui::ViewWallet>();
                }
              }

                _ => {}
              }
            });
          }

      State::EnterKey(result_message) => {
        let mut is_valid = false;
        let error_message = i18n("Invalid input detected. Please enter a valid seed phrase (12 or 24 words)");

        let action = render_centered_content(ctx, ui, i18n("Key/Seed Entry"), 200.0, |ui| {

          if result_message.len() > 0 {
            ui.heading(result_message);
          }

          let response = ui.add(
            TextEdit::multiline(&mut self.import_key_input)
              .desired_width(400.0)
              .desired_rows(6)
              .return_key(None)
              .hint_text(i18n("Enter your existing seed phrase")),
          );
          self.next_focus(ui, Focus::SeedPhrase, response.to_owned());

          let words: Vec<&str> = self.import_key_input
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .collect();
          let word_count = words.len();

          is_valid = (word_count == 12 || word_count == 24)
            && words.iter().all(|word| word.chars().all(|c| c.is_alphabetic()));

          if word_count > 0 && !is_valid {
            ui.colored_label(egui::Color32::RED, error_message);
          } else {
            ui.label("");
          }

          if 
            (response.lost_focus() && handle_enter_key(ui) && is_valid) ||
            ui.dx_large_button_enabled(is_valid, i18n("Import Wallet")).clicked() 
          {
            return WizardAction::Next(State::ImportKey {
              is_pending: Arc::new(Mutex::new(false)), 
              result: Arc::new(Mutex::new(None)) 
            });
          }

          WizardAction::NoAction
        });

        match action {
          WizardAction::Back => {
            self.state = State::WalletSecrets;
            self.import_key_input = String::new();
          },
          WizardAction::Next(next_state) => self.state = next_state,
          WizardAction::NoAction => {},
        }
      }

      State::ImportKey { is_pending, result } => {
        let mnemonic = self.import_key_input.to_owned();
        let is_pending = is_pending.clone();
        // Display the "Creating Wallet" UI
        render_centered_content_noback(ctx, ui, i18n("Importing Wallet"), 200.0, |ui| {
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
      
          let args_create = self.args_create.to_owned();
          let wallet = self.manager.wallet().to_owned();
          let result = result.clone();
      
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
                wallet.to_owned().batch().await?;

                let wallet_args = WalletCreateArgs::new(
                  args_create.wallet_name.is_not_empty().then_some(args_create.wallet_name),
                  args_create.wallet_filename.is_not_empty().then_some(args_create.wallet_filename),
                  EncryptionKind::XChaCha20Poly1305,
                  args_create.enable_phishing_hint.then_some(args_create.phishing_hint.into()),
                  false,
                );

                wallet.to_owned().wallet_create(wallet_secret.to_owned(), wallet_args).await?;

                let prv_key_data_args = PrvKeyDataCreateArgs::new(
                  None,
                  payment_secret.to_owned(),
                  Secret::from(mnemonic.trim()),
                );

                let prv_key_data_id = wallet.to_owned()
                  .prv_key_data_create(wallet_secret.to_owned(), prv_key_data_args)
                  .await?;

                let account_name = args_create.account_name.to_owned();

                let account_create_args = AccountCreateArgs::new_bip32(
                  prv_key_data_id,
                  payment_secret.to_owned(),
                  Some(account_name),
                  None,
                );

                let account_descriptor = wallet.to_owned()
                  .accounts_create(wallet_secret.to_owned(), account_create_args)
                  .await?;

                wallet.to_owned().flush(wallet_secret).await?;
                
                Ok(account_descriptor)
              }
            }
            .await;

            *result.lock().unwrap() = Some(res);
          });
        }

        // Check if the asynchronous task has completed
        if let Some(res) = result.lock().unwrap().take() {
          match res {
            Ok(account_descriptor) => {
              self.args_create.zeroize();
              self.import_key_input = String::new();
              self.assign_focus(Focus::None);
              core.handle_account_creation(vec![account_descriptor]);
              core.set_active_component::<components::wallet_ui::ViewWallet>();
            }
            Err(err) => {
              log_error!("{} {}", i18n("Wallet import error:"), err);
              self.state = State::EnterKey(
                format!("{} {}", i18n("Wallet import error:"), err).to_string(),
              );
            }
          }
        }
      }
      _ => {}
    }
  }
}