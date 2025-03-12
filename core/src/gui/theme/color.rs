use crate::imports::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccentImgID {
  Paw,
  Snow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ThemeColor {
  pub name: String,
  pub dark_mode: bool,

  pub hyperlink_color: Color32,
  pub node_data_color: Color32,
  pub balance_color: Color32,
  pub balance_syncing_color: Color32,
  pub error_color: Color32,
  pub alert_color: Color32,
  pub warning_color: Color32,
  pub info_color: Color32,

  pub qr_background: Color32,
  pub qr_foreground: Color32,

  pub default_color: Color32,
  pub strong_color: Color32,

  pub separator_color: Color32,
  pub button_color: Color32,
  pub toggle_active: Color32,

  pub bg_color: Color32,
  pub fg_color: Color32,

  pub null_balance_color: Color32,

  pub text_off_color_1: Color32,
  pub text_off_color_2: Color32,
  pub text_on_color_1: Color32,
  pub text_on_color_2: Color32,

  pub disabled_a: u8,
  pub accent_img: AccentImgID,
}

impl ThemeColor {
  pub fn pink() -> Self {
    Self {
      name: "WagLayla".to_string(),
      dark_mode: true,
      hyperlink_color: Color32::from_rgb(255, 105, 180),

      default_color: Color32::from_rgb(255, 202, 228),
      strong_color: Color32::WHITE,

      separator_color: Color32::from_rgb(69, 77, 99),
      button_color: Color32::from_rgb(14, 22, 42),
      toggle_active: Color32::from_rgb(223, 117, 141),

      bg_color: Color32::from_rgb(223, 117, 141),
      fg_color: Color32::from_rgb(29, 36, 55),

      null_balance_color: Color32::from_rgb(251, 142, 165),

      node_data_color: Color32::from_rgb(255, 192, 203),
      balance_color: Color32::from_rgb(255, 240, 245),
      balance_syncing_color: Color32::from_rgb(255, 182, 193),
      error_color: Color32::from_rgb(255, 99, 71),
      alert_color: Color32::from_rgb(255, 69, 0),
      warning_color: Color32::from_rgb(255, 140, 0),
      info_color: Color32::from_rgb(255, 218, 185),

      qr_background: Color32::from_rgba(255, 240, 245, 0),
      qr_foreground: Color32::from_rgb(255, 105, 180),

      text_off_color_1: Color32::from_rgb(14, 22, 42),
      text_off_color_2: Color32::from_rgb(4, 7, 12),
      text_on_color_1: Color32::WHITE,
      text_on_color_2: Color32::from_rgb(255, 175, 210),

      disabled_a: 95,
      accent_img: AccentImgID::Paw,
    }
  }

  pub fn dark() -> Self {
    Self {
      name: i18n("Carbon").to_string(),
      dark_mode: true,
      hyperlink_color: Color32::from_rgb(141, 184, 178),

      default_color: Color32::from_rgb(164, 164, 164),
      strong_color: Color32::from_rgb(223, 117, 141),

      separator_color: Color32::from_rgb(60, 60, 60),
      button_color: Color32::from_rgb(13, 13, 13),
      toggle_active: Color32::from_rgb(223, 117, 141),

      bg_color: Color32::from_rgb(13, 13, 13),
      fg_color: Color32::from_rgb(27, 27, 27),

      null_balance_color: Color32::from_rgb(22, 22, 22),

      node_data_color: Color32::WHITE,
      balance_color: Color32::WHITE,
      balance_syncing_color: Color32::DARK_GRAY,
      error_color: Color32::from_rgb(255, 136, 136),
      alert_color: Color32::from_rgb(255, 136, 136),
      warning_color: egui::Color32::from_rgb(255, 255, 136),
      info_color: egui::Color32::from_rgb(66, 178, 252),

      qr_background: Color32::from_rgba(0, 0, 0, 0),
      qr_foreground: Color32::WHITE,

      text_off_color_1: Color32::from_rgb(96, 96, 96),
      text_off_color_2: Color32::from_rgb(27, 27, 27),
      text_on_color_1: Color32::from_rgb(223, 117, 141),
      text_on_color_2: Color32::from_rgb(137, 137, 137),

      disabled_a: 95,
      accent_img: AccentImgID::Paw,
    }
  }

  pub fn light() -> Self {
    Self {
      name: i18n("Sakura").to_string(),
      dark_mode: false,
      hyperlink_color: Color32::from_rgb(15, 84, 73),

      default_color: Color32::from_rgb(255, 99, 160),
      strong_color: Color32::from_rgb(239, 42, 139),

      separator_color: Color32::from_rgb(255, 255, 255),
      button_color: Color32::from_rgb(255, 227, 244),
      toggle_active: Color32::from_rgb(255, 99, 160),

      bg_color: Color32::WHITE,
      fg_color: Color32::from_rgb(255, 196, 225),

      null_balance_color: Color32::from_rgb(255, 232, 238),

      node_data_color: Color32::BLACK,
      balance_color: Color32::BLACK,
      balance_syncing_color: Color32::LIGHT_GRAY,
      error_color: Color32::from_rgb(255, 69, 0),
      alert_color: Color32::from_rgb(255, 69, 0),
      warning_color: egui::Color32::from_rgb(77, 77, 41),
      info_color: egui::Color32::from_rgb(41, 56, 77),

      qr_background: Color32::from_rgba(255, 255, 255, 0),
      qr_foreground: Color32::BLACK,

      text_off_color_1: Color32::from_rgb(255, 196, 225),
      text_off_color_2: Color32::LIGHT_GRAY,
      text_on_color_1: Color32::from_rgb(239, 42, 139),
      text_on_color_2: Color32::DARK_GRAY,

      disabled_a: 95,
      accent_img: AccentImgID::Paw,
    }
  }

  pub fn snow() -> Self {
    Self {
      name: i18n("Arctic").to_string(),
      dark_mode: false,
      hyperlink_color: Color32::from_rgb(15, 84, 73),

      default_color: Color32::from_rgb(60, 151, 194),
      strong_color: Color32::from_rgb(2, 130, 184),

      separator_color: Color32::from_rgb(235, 238, 240),
      button_color: Color32::from_rgb(192, 236, 252),
      toggle_active: Color32::from_rgb(33, 184, 255),

      bg_color: Color32::from_rgb(192, 236, 252),
      fg_color: Color32::from_rgb(249, 254, 255),

      null_balance_color: Color32::from_rgb(232, 250, 255),

      node_data_color: Color32::BLACK,
      balance_color: Color32::BLACK,
      balance_syncing_color: Color32::LIGHT_GRAY,
      error_color: Color32::from_rgb(255, 69, 0),
      alert_color: Color32::from_rgb(255, 69, 0),
      warning_color: egui::Color32::from_rgb(77, 77, 41),
      info_color: egui::Color32::from_rgb(41, 56, 77),

      qr_background: Color32::from_rgba(255, 255, 255, 0),
      qr_foreground: Color32::BLACK,

      text_off_color_1: Color32::from_rgb(242, 252, 255),
      text_off_color_2: Color32::from_rgb(242, 252, 255),
      text_on_color_1: Color32::from_rgb(2, 130, 184),
      text_on_color_2: Color32::from_rgb(2, 130, 184),

      disabled_a: 55,
      accent_img: AccentImgID::Snow,
    }
  }
}

impl Default for ThemeColor {
  fn default() -> Self {
    Self::pink()
  }
}

impl ThemeColor {
  pub fn name(&self) -> &str {
    &self.name
  } 
}

static mut THEME_COLOR_LIST: Option<HashMap<String, ThemeColor>> = None;
pub fn theme_colors() -> &'static HashMap<String, ThemeColor> {
  unsafe {
    THEME_COLOR_LIST.get_or_insert_with(|| {
      let mut themes = HashMap::new();
      [ThemeColor::pink(), ThemeColor::dark(), ThemeColor::light(), ThemeColor::snow()]
        .into_iter()
        .for_each(|theme| {
          themes.insert(theme.name.clone(), theme.clone());
        });
      themes
    })
  }
}