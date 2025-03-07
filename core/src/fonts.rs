use egui::{FontData, FontDefinitions, FontFamily};
use workflow_core::runtime;

trait RegisterStaticFont {
  fn add_static(&mut self, family: FontFamily, name: &str, bytes: &'static [u8]);
}

impl RegisterStaticFont for FontDefinitions {
  fn add_static(&mut self, family: FontFamily, name: &str, bytes: &'static [u8]) {
    self.font_data
      .insert(name.to_owned(), FontData::from_static(bytes).into());

    self.families
      .entry(family)
      .or_default()
      .push(name.to_owned());
  }
}

use egui_phosphor::Variant;
pub fn add_to_fonts(fonts: &mut egui::FontDefinitions, variant: Variant) {
  let variant_name = match variant {
    Variant::Thin => "phosphor-thin",
    Variant::Light => "phosphor-light",
    Variant::Regular => "phosphor",
    Variant::Bold => "phosphor-bold",
    Variant::Fill => "phosphor-fill",
  };

  let mut font_data = variant.font_data();
  
  font_data.tweak.y_offset_factor = 0.0;
  font_data.tweak.y_offset = 0.0;

  fonts.font_data.insert(variant_name.to_string(), font_data);

  fonts
    .families
    .entry(egui::FontFamily::Name(variant_name.into()))
    .or_default()
    .insert(0, variant_name.to_owned());

  if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
    font_keys.push(variant_name.to_owned());
  }
}

pub fn get_font_family(base_name: &str, bold: bool, italic: bool) -> egui::FontFamily {
  let mut full_name = String::from(base_name);
  
  // Check if we need to modify the name
  if bold || italic {
    full_name.push('-');
    
    if bold {
      full_name.push_str("Bold");
    }
    
    if italic {
      full_name.push_str("Italic");
    }
  }
  
  egui::FontFamily::Name(full_name.into())
}

macro_rules! load_font_family {
  ($fonts:expr, $base_name:literal, $($variant:ident),+) => {{      
    $(
      let variant_suffix = stringify!($variant);
      let name = if variant_suffix == "Regular" {
        format!("{}", $base_name)
      } else {
        format!("{}-{}", $base_name, variant_suffix)
      };
      
      $fonts.font_data.insert(
        name.clone(),
        FontData::from_static(include_bytes!(
          concat!("../resources/fonts/", $base_name, "/", $base_name, "-", stringify!($variant), ".ttf")
        )).into()
      );
      $fonts.families
      .entry(FontFamily::Name(name.clone().into()))
      .or_default()
      .insert(0, name.into());
    )+
  }};
}




pub fn init_fonts(cc: &eframe::CreationContext<'_>) {
  let mut fonts = FontDefinitions::default();
  add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
  add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
  add_to_fonts(&mut fonts, egui_phosphor::Variant::Fill);
  add_to_fonts(&mut fonts, egui_phosphor::Variant::Light);

  // ---
  load_font_family!(fonts, "DINishCondensed", Regular, Bold, Italic, BoldItalic);
  let variants = ["", "-Bold", "-Italic", "-BoldItalic"];
  for variant in variants {
    let font_name = format!("{}{}", "DINishCondensed", variant);
    if let Some(font_data) = fonts.font_data.get_mut(&font_name) {
      font_data.tweak.y_offset_factor = 0.0;
      font_data.tweak.y_offset = -3.0;
    }
  }

  load_font_family!(fonts, "DINish", Regular, Bold, Italic, BoldItalic);
  let variants = ["", "-Bold", "-Italic", "-BoldItalic"];
  for variant in variants {
    let font_name = format!("{}{}", "DINish", variant);
    if let Some(font_data) = fonts.font_data.get_mut(&font_name) {
      font_data.tweak.y_offset_factor = 0.0;
      font_data.tweak.y_offset = -3.0;
    }
  }
  // ---

  fonts
    .families
    .entry(FontFamily::Monospace)
    .or_default()
    .insert(0, "ubuntu_mono".to_owned());

  fonts.font_data.insert(
    "ubuntu_mono".to_owned(),
    egui::FontData::from_static(include_bytes!(
      "../resources/fonts/UbuntuMono/UbuntuMono-Regular.ttf"
    )).into(),
  );
  // ---

  fonts.font_data.insert(
    "noto_sans_mono_light".to_owned(),
    FontData::from_static(include_bytes!(
      "../resources/fonts/NotoSans/NotoSansMono-Light.ttf"
    )).into(),
  );

  fonts
    .families
    .entry(egui::FontFamily::Name("noto_sans_mono_light".into()))
    .or_default()
    .insert(0, "noto_sans_mono_light".to_owned());

  // ---

  #[cfg(target_os = "linux")]
  if let Ok(font) = std::fs::read("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc") {
    fonts
      .font_data
      .insert("noto-sans-cjk".to_owned(), egui::FontData::from_owned(font));

    fonts
      .families
      .entry(egui::FontFamily::Proportional)
      .or_default()
      .push("noto-sans-cjk".to_owned());
  }

  // ---

  if runtime::is_native() || runtime::is_chrome_extension() {
    fonts.add_static(
      FontFamily::Proportional,
      "ar",
      include_bytes!(
        // "../resources/fonts/NotoSansArabic/NotoSansArabic-Light.ttf"
        "../resources/fonts/NotoSansArabic/NotoSansArabic-Regular.ttf"
      ),
    );

    fonts.add_static(
      FontFamily::Proportional,
      "he",
      include_bytes!(
        // "../resources/fonts/NotoSansHebrew/NotoSansHebrew-Light.ttf"
        "../resources/fonts/NotoSansHebrew/NotoSansHebrew-Regular.ttf"
      ),
    );

    fonts.add_static(
      FontFamily::Proportional,
      "ja",
      include_bytes!(
        // "../resources/fonts/NotoSansJP/NotoSansJP-Light.ttf"
        "../resources/fonts/NotoSansJP/NotoSansJP-Regular.ttf"
      ),
    );

    fonts.add_static(
      FontFamily::Proportional,
      "hi",
      include_bytes!(
        // "../resources/fonts/NotoSansJP/NotoSansJP-Light.ttf"
        "../resources/fonts/NotoSansDevanagari/NotoSansDevanagari-Regular.ttf"
      ),
    );

    fonts.add_static(
      FontFamily::Proportional,
      "zh",
      include_bytes!(
        // "../resources/fonts/NotoSansSC/NotoSansSC-Light.ttf"
        "../resources/fonts/NotoSansSC/NotoSansSC-Regular.ttf"
      ),
    );

    fonts.add_static(
      FontFamily::Proportional,
      "ko",
      include_bytes!(
        // "../resources/fonts/NotoSansKR/NotoSansKR-Light.ttf"
        "../resources/fonts/NotoSansKR/NotoSansKR-Regular.ttf"
      ),
    );
  }

  cc.egui_ctx.set_fonts(fonts);
}
