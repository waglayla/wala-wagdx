use super::*;

define_animation_frames!(WALA_WAG, 192, "/resources/animation/layla/wag");

pub struct Outline {
  selected_tab: Tab,
  account_dropdown_open: bool,
  wala_animation: Option<SpriteAnimation>,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum Tab {
  Wallet,
  NetworkInfo,
  WalaNode,
  WalaBridge,
  Donate,
  About,
}

impl Tab {
  fn label(&self) -> &'static str {
    match self {
      Tab::Wallet => i18n("Wallet"),
      Tab::NetworkInfo => i18n("Network"),
      Tab::WalaNode => i18n("Node"),
      Tab::WalaBridge => i18n("Stratum"),
      Tab::Donate => i18n("Donate"),
      Tab::About => i18n("About"),
    }
  }

  fn component(&self) -> TypeId {
    match self {
      Tab::Wallet => TypeId::of::<wallet_ui::WalletDelegator>(),
      Tab::NetworkInfo => TypeId::of::<network::NetworkInfo>(),
      Tab::WalaNode => TypeId::of::<console::DaemonConsole>(),
      Tab::WalaBridge => TypeId::of::<bridge::StratumBridge>(),
      Tab::Donate => TypeId::of::<donate::Donate>(),
      Tab::About => TypeId::of::<about::About>(),
    }
  }
}

impl ComponentT for Outline {
  fn name(&self) -> Option<&'static str> {
    Some("Outline")
  }

  fn render(
    &mut self,
    core: &mut Core,
    ctx: &egui::Context,
    _frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
  ) {
    let panel_fill = theme_color().fg_color;
    let darkened_fill = theme_color().bg_color;

    egui::SidePanel::left("sidebar")
      .exact_width(225.0)
      .resizable(false)
      .show_separator_line(false)
      .frame(egui::Frame {
        fill: darkened_fill, 
        inner_margin: egui::Margin::ZERO,
        outer_margin: egui::Margin::ZERO,
        ..Default::default()
      })
      .show_inside(ui, |ui| {
        ui.set_min_height(ui.available_height());
        ui.spacing_mut().item_spacing = Vec2::ZERO;

        let info_space = ui.allocate_space(egui::Vec2::new(ui.available_width(), 132.0));
        let info_rect = info_space.1;
        
        self.render_platform(ui, core, &info_rect);
        self.render_layla(ui, core, &info_rect);

        self.render_account_section(ui, core, &info_rect);
        self.render_balance_section(ui, core, &info_rect);

        ui.style_mut().text_styles.insert(
          egui::TextStyle::Button,
          egui::FontId::new(30.0, get_font_family("DINishCondensed", false, false))
        );
        
        for tab in self.available_tabs(core) {
          if self.tab_button(ui, ctx, tab) {
            self.selected_tab = tab;
            core.set_active_component_by_type(tab.component().to_owned());
          }
        }
      });
  }
}

impl Outline {
  pub fn new(ctx: &egui::Context) -> Self {
    let wala_animation = Some(
      SpriteAnimationBuilder::new()
        .fps(30.0)
        .looping(true)
        .build(ctx, &WALA_WAG)
    );

    Self {
      selected_tab: Tab::iter().next().unwrap(),
      account_dropdown_open: false,
      wala_animation,
    }
  }

  fn account_button(
    &self,
    ui: &mut Ui,
    text: &str,
    size: Vec2,
  ) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
    response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    
    let color = Color32::TRANSPARENT;

    if ui.is_rect_visible(rect) {
      ui.painter().rect_filled(
        rect,
        Rounding::ZERO,
        color,
      );

      let size_factor = ui.ctx().animate_value_with_time(
        response.id.with("text_size"),
        if response.hovered() { 1.05 } else { 1.0 },
        0.075,
      );

      let text_padding = 12.0;
      let text_rect = rect.shrink2(vec2(text_padding, 0.0));
      
      let text_color = if response.hovered() {
        theme_color().strong_color
      } else {
        theme_color().default_color
      };

      let font_id = FontId::new(
        26.0 * size_factor,
        get_font_family("DINishCondensed", true, false)
      );

      ui.painter().text(
        text_rect.left_top() + vec2(0.0, 24.0),
        egui::Align2::LEFT_CENTER,
        text,
        font_id,
        text_color,
      );
    }

    response
  }

  fn render_account_dropdown(&mut self, ui: &mut Ui, core: &Core, account_rect: Rect) {
    let area_id = ui.make_persistent_id("account_dropdown_area");
    let layer_id = egui::LayerId::new(egui::Order::Foreground, area_id);
    
    let mut dropdown_rect = egui::Rect::from_min_max(
      egui::pos2(account_rect.min.x, account_rect.max.y + 20.0),
      egui::pos2(account_rect.max.x, account_rect.max.y + 200.0),
    );
    
    if let screen_rect = ui.ctx().screen_rect() {
      dropdown_rect.max.y = dropdown_rect.max.y.min(screen_rect.max.y);
    }

    if ui.input(|i| i.pointer.any_click()) {
      if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
        let clicking_account_selector = account_rect.contains(pos);
        let clicking_dropdown = dropdown_rect.contains(pos);
        
        // Get the ID of the widget being clicked, if any
        let clicked_id = ui.ctx().layer_id_at(pos).map(|id| id.id);
        
        // Check if the click is within our dropdown layer
        let clicking_dropdown_contents = clicked_id == Some(area_id);

        if !clicking_account_selector && !clicking_dropdown && !clicking_dropdown_contents {
          self.account_dropdown_open = false;
          return;
        }
      }
    }


    let area = egui::Area::new(area_id)
      .fixed_pos(dropdown_rect.min)
      .order(egui::Order::Foreground);

    area.show(ui.ctx(), |ui| {
      let dropdown_frame = egui::Frame {
        fill: theme_color().button_color,
        rounding: egui::Rounding::same(4.0),
        shadow: dx_shadow(),
        ..Default::default()
      };

      dropdown_frame.show(ui, |ui| {
        ui.set_min_width(dropdown_rect.width());

        let mut core_delegate = core.to_owned();
        let current_account_delegate = core_delegate.to_owned().current_account.unwrap();

        ScrollArea::vertical()
          .max_height(220.0)
          .show(ui, |ui| {
            ui.style_mut().spacing.item_spacing = Vec2::ZERO;
            
            let account_list = if let Some(account_collection) = core_delegate.to_owned().user_accounts() {
              account_collection.list().to_owned()
            } else {
              return;
            };
            account_list.iter().for_each(|account|{
              if account.id() != current_account_delegate.id() {
                let account_name = account.name_or_id().to_string();
                if self.account_button(ui, &account_name, vec2(220.0, 40.0)).clicked() {
                  core_delegate.select_account(Some(account.to_owned()), true);
                  self.account_dropdown_open = false;
                }
              }
            });
          });

        ui.separator();
        
        if self.account_button(ui, i18n("Add Account"), vec2(220.0, 40.0))
          .on_hover_cursor(egui::CursorIcon::Help)
          .on_hover_text_at_pointer(i18n("Feature Coming Soon..."))
          .clicked() {
          // Handle new account creation
          // self.account_dropdown_open = false;
        }
        ui.add_space(6.0);
      });
    });
  }

  fn render_platform(&mut self, ui: &mut Ui, core: &Core, info_rect: &Rect) {
    if let Some(animation) = &mut self.wala_animation {
      let platform_pos = egui::Pos2 {
        x: info_rect.max.x - 54.0,
        y: info_rect.min.y + 10.0,
      };

      match theme_color().name().to_string().as_str() {
        s if s == i18n("Arctic") => {
          DXImage::paint_at(ui, &Assets::get().snow_platform, 107.0, platform_pos, egui::Align2::CENTER_TOP);
        },
        s if s == i18n("Meadow") => {
          DXImage::paint_at(ui, &Assets::get().tree, 92.0, platform_pos + vec2(15.0, -9.0), egui::Align2::CENTER_TOP);
        }
        _ => {}
      }
    }
  }

  fn render_layla(&mut self, ui: &mut Ui, core: &Core, info_rect: &Rect) {
    if let Some(animation) = &mut self.wala_animation {
      let animation_pos = egui::Pos2 {
        x: info_rect.max.x - 92.0,
        y: info_rect.min.y + 5.0,
      };

      if !animation.is_animating() {
        animation.animate();
      }

      animation.paint(ui, animation_pos, 0.46);
    }
  }

  fn render_account_section(&mut self, ui: &mut Ui, core: &Core, info_rect: &Rect) {
    let painter = ui.painter_at(*info_rect);
    let account_pos = egui::Pos2 {
      x: info_rect.min.x + 11.5,
      y: info_rect.min.y + 12.0,
    };

    let has_account = core.current_account.is_some();

    let account_name_base = core.current_account.as_ref().map_or_else(
      || i18n("No Account").to_string(),
      |account| account.name_or_id(),
    );
    
    let truncated_name = if account_name_base.chars().count() > 10 {
      format!("{}...", &account_name_base.chars().take(7).collect::<String>())
    } else {
      account_name_base.clone()
    };

    let account_name = format!("• {}", truncated_name);

    let account_rect = egui::Rect::from_min_max(
      info_rect.min,
      egui::pos2(info_rect.max.x - 60.0, info_rect.min.y + 30.0),
    );

    let mut account_response = ui.interact(
      account_rect,
      egui::Id::new("account_selector"),
      egui::Sense::click(),
    );

    if has_account {
      account_response = account_response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }

    let acc_color = if has_account && account_response.hovered() {
      theme_color().strong_color
    } else {
      theme_color().text_off_color_1
    };

    let color = ui.ctx().animate_color_with_time(
      account_response.id.with("account_title_color"),
      acc_color,
      0.075
    );

    painter.text(
      account_pos,
      egui::Align2::LEFT_TOP,
      account_name,
      egui::FontId::new(26.0, get_font_family("DINishCondensed", true, false)),
      color,
    );

    if has_account {
      if account_response.clicked() {
        self.account_dropdown_open = !self.account_dropdown_open;
      }

      if self.account_dropdown_open {
        self.render_account_dropdown(ui, core, account_rect);
      }
    }
  }

  fn render_balance_section(&self, ui: &mut Ui, core: &Core, info_rect: &Rect) {
    let painter = ui.painter_at(*info_rect);
    
    let whole_pos = egui::Pos2 {
      x: info_rect.min.x + 127.,
      y: info_rect.max.y + 16.,
    };
    let part_pos = egui::Pos2 {
      x: info_rect.min.x + 127.5,
      y: info_rect.max.y - 2.8,
    };
    let sym_pos = egui::Pos2 {
      x: info_rect.max.x - 10.75,
      y: info_rect.max.y - 6.,
    };

    let account_clone = core.current_account.clone();

    let mut pad_str = "".to_string();
    let mut big_str = "000".to_string();
    let mut small_str = ".000".to_string();
    let symbol_color = theme_color().text_off_color_1;
    let fade_color = theme_color().null_balance_color;
    let mut balance_color = fade_color;

    if let Some(ref account) = account_clone {
      let balance = account.balance().unwrap_or_default();
      let (padded, whole, partial) = format_balance_with_precision(balance.mature);
      pad_str = padded;
      big_str = whole;
      small_str = partial;
      balance_color = theme_color().strong_color;
    }

    let balance_rect = egui::Rect::from_min_max(
      egui::Pos2 {
        x: whole_pos.x - 92.0,
        y: whole_pos.y - 92.0,
      },
      egui::Pos2 {
        x: whole_pos.x + 30.0,
        y: whole_pos.y,
      },
    );

    let response = ui.interact(balance_rect, egui::Id::new("balance_area"), egui::Sense::hover())
      .on_hover_cursor(egui::CursorIcon::Help);

    painter.text(
      whole_pos,
      egui::Align2::RIGHT_BOTTOM,
      pad_str,
      egui::FontId::new(92.0, get_font_family("DINishCondensed", true, false)),
      fade_color,
    );

    painter.text(
      whole_pos,
      egui::Align2::RIGHT_BOTTOM,
      big_str,
      egui::FontId::new(92.0, get_font_family("DINishCondensed", true, false)),
      balance_color,
    );

    painter.text(
      part_pos,
      egui::Align2::LEFT_BOTTOM,
      small_str,
      egui::FontId::new(25.0, get_font_family("DINishCondensed", true, false)),
      balance_color,
    );

    painter.text(
      sym_pos,
      egui::Align2::RIGHT_BOTTOM,
      "WALA",
      egui::FontId::new(16.0, get_font_family("DINish", false, false)),
      symbol_color,
    );

    if let Some(ref account) = account_clone {
      response.on_hover_text_at_pointer(format!(
        "{} {} WALA ({} {})",
        i18n("Current Balance:"),
        format_balance(
          account.balance().unwrap_or_default().mature,
        ),
        format_balance(
          account.balance().unwrap_or_default().pending,
        ),
        i18n("Pending"),
      ));
    }
  }

  fn tab_button(&self, ui: &mut Ui, ctx: &Context, tab: Tab) -> bool {
    let panel_fill = theme_color().fg_color;
    let selected = self.selected_tab == tab;
    
    let mut visuals = ui.style_mut().visuals.clone();
    let bg_color = if selected {
      panel_fill
    } else {
      Color32::TRANSPARENT
    };
  
    visuals.widgets.inactive.weak_bg_fill = bg_color;
    visuals.widgets.active.weak_bg_fill = bg_color;
    ui.style_mut().visuals = visuals;
  
    let button_size = vec2(ui.available_width(), 55.0);
    let (rect, mut response) = ui.allocate_exact_size(button_size, egui::Sense::click());
    if !selected {          
      response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
  
    let color = ctx.animate_color_with_time(
      response.id.with("outline_active_color"),
      bg_color,
      0.075
    );
  
    if ui.is_rect_visible(rect) {
      ui.spacing_mut().item_spacing = Vec2::ZERO;
      
      ui.painter().rect_filled(
          rect,
          Rounding::ZERO,
          color,
      );

      let size_factor = ctx.animate_value_with_time(
          response.id.with("text_size"),
          if response.hovered() { 1.05 } else { 1.0 },
          0.075,
      );

      let text_padding = 12.0;
      let text_rect = rect.shrink2(vec2(text_padding, 0.0));
      let base_color = theme_color().text_off_color_1;

      let luminance = get_luminance(theme_color().bg_color);
      
      let text_color = if selected {
          theme_color().text_on_color_1
      } else if response.hovered() {
        if luminance > 0.5 {
          base_color.linear_multiply_rgb(0.55)
        } else {
          base_color.linear_multiply_rgb(1.8)
        }
      } else {
          theme_color().text_off_color_1
      };

      let mut font_id = ui.style().text_styles[&egui::TextStyle::Button].clone();
      font_id.size *= size_factor;
  
      ui.painter().text(
        text_rect.left_center() + vec2(0.0, 3.0),
        egui::Align2::LEFT_CENTER,
        tab.label(),
        font_id,
        text_color,
      );
    }
  
    response.clicked()
  }

  fn available_tabs(&self, core: &Core) -> Vec<Tab> {
    let mut tabs = vec![Tab::Wallet];

    if core.settings.node.node_kind == WagLayladNodeKind::IntegratedAsDaemon {
      #[cfg(not(target_arch = "wasm32"))]
      tabs.push(Tab::WalaBridge);
      
      tabs.push(Tab::WalaNode);
    }

    tabs.push(Tab::NetworkInfo);
    tabs.push(Tab::Donate);
    tabs.push(Tab::About);
    tabs
  }
}