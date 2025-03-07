use crate::imports::*;
use std::sync::{Arc, Mutex};
use core::cmp::max;

 // wtf rust why do I need this

#[derive(Clone, Default)]
pub enum State {
  #[default]
  Idle,
  Unlock { wallet_descriptor : WalletDescriptor, error : Option<String>},
}

define_indexed_enum!(
  Focus,
  WalletSecret
);

pub struct OpenWallet {
  #[allow(dead_code)]
  manager: DX_Manager,
  wallet_secret: String,
  pub state: State,
  pub message: Option<String>,
  pub unlock_result: Arc<Mutex<Option<UnlockResult>>>,
  pub is_pending: Arc<Mutex<bool>>,
  focus_context: FocusContext,
}

impl_has_focus_context!(OpenWallet);

impl OpenWallet {
  pub fn new(manager: DX_Manager) -> Self {
    Self {
      manager,
      wallet_secret: String::new(),
      state: State::Idle,
      message: None,
      unlock_result: Arc::new(Mutex::new(None)),
      is_pending: Arc::new(Mutex::new(false)),
      focus_context: FocusContext { focus: Focus::WalletSecret.to_u32().unwrap() },
    }
  }

  pub fn open(&mut self, wallet_descriptor: WalletDescriptor) {
    self.state = State::Unlock { wallet_descriptor, error : None};
    self.assign_focus(Focus::WalletSecret);
  }
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

impl ComponentT for OpenWallet {
  fn render(
      &mut self,
      core: &mut Core,
      ctx: &egui::Context,
      _frame: &mut eframe::Frame,
      ui: &mut egui::Ui,
  ) {

    match self.state.clone() {
      State::Idle => {}
      State::Unlock { wallet_descriptor, error } => {
        let wallet_descriptor_delegate = wallet_descriptor.clone();

        let action = render_centered_content(ctx, ui, i18n("Unlock Wallet"), 180.0, |ui| {
          ui.vertical_centered(|ui| {
            ui.heading(format!(
              "{} \"{}\"",
              i18n("Opening"),
              wallet_descriptor
                .title
                .as_deref()
                .unwrap_or(wallet_descriptor_delegate.clone().filename.as_str())
            ));
            ui.add_space(8.);

            if let Some(err) = error {
              ui.label(
                egui::RichText::new(err)
                  .color(egui::Color32::from_rgb(255, 120, 120)),
              );
              ui.add_space(8.);
            }

            ui.label(i18n("Enter the password to unlock your wallet:"));
            ui.add_space(8.);

            let input_width = 300.0;
            let font_size = 20.0;
            let response = ui.add_sized(
              [input_width, 20.0],
              egui::TextEdit::singleline(&mut self.wallet_secret)
                .password(true)
                .hint_text(i18n("Enter password"))
                .font(egui::FontId::proportional(font_size)),
            );
            self.next_focus(ui, Focus::WalletSecret, response.clone());

            let enter = 
              !self.wallet_secret.trim().is_empty() && 
              response.lost_focus() && 
              handle_enter_key(ui)
            ;
              
            ui.add_space(20.);

            if 
              enter ||
              ui.dx_large_button_enabled(!self.wallet_secret.trim().is_empty(), i18n("Unlock")).clicked() 
            {
              if !*self.is_pending.lock().unwrap() {      
                *self.is_pending.lock().unwrap() = true;
                let wallet_secret = Secret::new(self.wallet_secret.as_bytes().to_vec());
                self.wallet_secret.zeroize();
                let wallet = self.manager.wallet().clone();
                let wallet_descriptor_delegate = wallet_descriptor.clone();
                let unlock_result_clone = Arc::clone(&self.unlock_result);
                let is_pending_clone = Arc::clone(&self.is_pending);
        
                tokio::spawn(async move {
                  let res = wallet
                    .wallet_open(
                      wallet_secret,
                      Some(wallet_descriptor_delegate.filename),
                      true,
                      true,
                    )
                    .await;
      
                  *unlock_result_clone.lock().unwrap() = Some(res);
                });
              }                
            }

            WizardAction::NoAction
          }).inner
      });

      match action {
        WizardAction::Next(next_state) => {
          self.state = next_state;
        }
        WizardAction::Back => {
          self.wallet_secret.zeroize();
          core.set_active_component::<components::wallet_ui::CreateWallet>();
        }
        _ => {}
      }

      if *self.is_pending.lock().unwrap() {
        let finish = false;
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

        let h_scale = max(450, (ui.available_width() / 1.5) as i32);
        let v_scale = max(450, (ui.available_height() / 1.33) as i32);
        let unlock_result_clone = Arc::clone(&self.unlock_result);

        egui::Window::new("Decrypting wallet")
          .collapsible(false)
          .resizable(false)
          .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
          .max_size([h_scale as f32, v_scale as f32])
          .frame(F)
          .show(ui.ctx(), |ui| {
            let available_height = ui.available_height();

            ui.vertical_centered(|ui| {
              ui.label("");
              ui.heading(i18n("Please wait..."));
              ui.add_space(24.);
  
              ui.add(DX_Spinner::new()
                .size(125.0)
                .color(theme_color().strong_color)
                .stroke_width(12.0)
              );
  
              ui.add_space(24.);
            });
      
            if let Some(result) = unlock_result_clone.lock().unwrap().take() {
              self.unlock_result = Arc::new(Mutex::new(None));
              self.is_pending = Arc::new(Mutex::new(false));
              match result {
                Ok(_) => {
                  core.set_active_component::<components::wallet_ui::ViewWallet>();
                  self.state = Default::default();
                }
                Err(err) => {
                  self.assign_focus(Focus::WalletSecret);
                  self.state = State::Unlock {
                    wallet_descriptor,
                    error: Some(err.to_string()),
                  };
                }
              }
            }
          });
        }
      }
    }
  }
}