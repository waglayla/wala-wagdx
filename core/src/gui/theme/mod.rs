use waglayla_metrics_core::MetricGroup;

mod color;
pub use color::*;
mod style;
pub use style::*;

use crate::imports::*;

#[derive(Clone)]
pub struct AppTheme {
  pub color: ThemeColor,
  pub style: ThemeStyle,
}

impl AppTheme {
  pub fn new(color: ThemeColor, style: ThemeStyle) -> Self {
    Self { color, style }
  }

  #[inline(always)]
  pub fn color(&self) -> &ThemeColor {
    &self.color
  }

  #[inline(always)]
  pub fn style(&self) -> &ThemeStyle {
    &self.style
  }
}

impl Default for AppTheme {
  fn default() -> Self {
    Self {
      color: ThemeColor::dark(),
      style: ThemeStyle::rounded(),
    }
  }
}

impl From<&AppTheme> for Visuals {
  fn from(theme: &AppTheme) -> Self {
    let mut visuals = if theme.color.dark_mode {
      Visuals::dark()
    } else {
      Visuals::light()
    };

    visuals.widgets.active.rounding = theme.style.widget_rounding;
    visuals.widgets.inactive.rounding = theme.style.widget_rounding;
    visuals.widgets.hovered.rounding = theme.style.widget_rounding;
    visuals.widgets.noninteractive.rounding = theme.style.widget_rounding;
    visuals.widgets.open.rounding = theme.style.widget_rounding;

    visuals.hyperlink_color = theme.color.hyperlink_color;
    visuals.warn_fg_color = theme.color.warning_color;
    visuals.error_fg_color = theme.color.error_color;

    visuals
  }
}

impl AsRef<AppTheme> for AppTheme {
  fn as_ref(&self) -> &Self {
    self
  }
}

static mut THEME: Option<AppTheme> = None;
#[inline(always)]
pub fn theme() -> &'static AppTheme {
  unsafe { THEME.get_or_insert_with(AppTheme::default) }
}

#[inline(always)]
pub fn theme_color() -> &'static ThemeColor {
  &theme().color
}

#[inline(always)]
pub fn theme_style() -> &'static ThemeStyle {
  &theme().style
}


#[inline(always)]
pub fn theme_accent_img() -> &'static TextureHandle {
    match &theme().color.accent_img {
        AccentImgID::Paw => &Assets::get().paw_watermark,
        AccentImgID::Snow => &Assets::get().snow_watermark,
    }
}

pub fn apply_theme_by_name(
  ctx: &Context,
  theme_color_name: impl Into<String>,
  theme_style_name: impl Into<String>,
) {
  let theme_color_name = theme_color_name.into();
  let theme_color = theme_colors()
    .get(&theme_color_name)
    .cloned()
    .unwrap_or_else(|| {
      log_error!("Theme color not found: {}", theme_color_name);
      ThemeColor::default()
    });

  let theme_style_name = theme_style_name.into();
  let theme_style = theme_styles()
    .get(&theme_style_name)
    .cloned()
    .unwrap_or_else(|| {
      log_error!("Theme style not found: {}", theme_style_name);
      ThemeStyle::default()
    });

  apply_theme(ctx, AppTheme::new(theme_color.clone(), theme_style));
}

pub fn apply_theme_color_by_name(ctx: &Context, theme_color_name: impl Into<String>) {
  let theme_color_name = theme_color_name.into();
  let theme_color = theme_colors()
    .get(&theme_color_name)
    .cloned()
    .unwrap_or_else(|| {
      log_error!("Theme not found: {}", theme_color_name);
      ThemeColor::default()
    });

  apply_theme(ctx, AppTheme::new(theme_color, theme_style().clone()));
}

pub fn apply_theme_style_by_name(ctx: &Context, theme_style_name: impl Into<String>) {
  let theme_style_name = theme_style_name.into();
  let theme_style = theme_styles()
    .get(&theme_style_name)
    .cloned()
    .unwrap_or_else(|| {
      log_error!("Theme not found: {}", theme_style_name);
      ThemeStyle::default()
    });

  apply_theme(ctx, AppTheme::new(theme_color().clone(), theme_style));
}

pub fn apply_theme(ctx: &Context, theme: AppTheme) {
  unsafe {
    THEME = Some(theme.clone());
  }
  ctx.set_visuals(theme.as_ref().into());
}

// ~

#[inline(always)]
pub fn error_color() -> Color32 {
  theme_color().error_color
}

#[inline(always)]
pub fn warning_color() -> Color32 {
  theme_color().warning_color
}

#[inline(always)]
pub fn info_color() -> Color32 {
  theme_color().info_color
}

#[inline(always)]
pub fn strong_color() -> Color32 {
  theme_color().strong_color
}