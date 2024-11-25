use egui::{FontData, FontTweak, FontDefinitions, FontFamily};
use std::path::Path;
use workflow_core::runtime;

trait RegisterStaticFont {
    fn add_static(&mut self, family: FontFamily, name: &str, bytes: &'static [u8]);
}

impl RegisterStaticFont for FontDefinitions {
    fn add_static(&mut self, family: FontFamily, name: &str, bytes: &'static [u8]) {
        self.font_data
            .insert(name.to_owned(), FontData::from_static(bytes));

        self.families
            .entry(family)
            .or_default()
            .push(name.to_owned());
    }
}

use egui_phosphor::Variant;
pub fn add_to_fonts(fonts: &mut egui::FontDefinitions, variant: Variant) {
    fonts
        .font_data
        .insert("phosphor".into(), variant.font_data());

    if let Some(font_keys) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        // font_keys.insert(0, "phosphor".into());
        font_keys.push("phosphor".into());
    }

    fonts
        .families
        .entry(egui::FontFamily::Name("phosphor".into()))
        .or_default()
        .insert(0, "phosphor".to_owned());
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
              ))
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
    // add_to_fonts(&mut fonts, egui_phosphor::Variant::Bold);
    // add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    add_to_fonts(&mut fonts, egui_phosphor::Variant::Light);

    // ---
    load_font_family!(fonts, "DINishCondensed", Regular, Bold, Italic, BoldItalic);
    let variants = ["", "-Bold", "-Italic", "-BoldItalic"];
    for variant in variants {
        let font_name = format!("{}{}", "DINishCondensed", variant);
        if let Some(font_data) = fonts.font_data.get_mut(&font_name) {
            font_data.tweak.y_offset_factor = 0.0;
            font_data.tweak.y_offset = 9.0;
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
        )),
    );
    // ---

    fonts.font_data.insert(
        "noto_sans_mono_light".to_owned(),
        FontData::from_static(include_bytes!(
            "../resources/fonts/NotoSans/NotoSansMono-Light.ttf"
        )),
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

    // ---

    // #[cfg(windows)]
    // {
    //     let font_file = {
    //         // c:/Windows/Fonts/msyh.ttc
    //         let mut font_path = PathBuf::from(std::env::var("SystemRoot").ok()?);
    //         font_path.push("Fonts");
    //         font_path.push("msyh.ttc");
    //         font_path.to_str()?.to_string().replace("\\", "/")
    //     };
    //     Some(font_file)
    // }

    // "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"
    // "/System/Library/Fonts/Hiragino Sans GB.ttc"

    // ---
    // fonts.font_data.insert(
    //     "test_font".to_owned(),
    //     // egui::FontData::from_static(include_bytes!("../../resources/fonts/NotoSans-Regular.ttf")),
    //     egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
    // );

    // fonts
    //     .families
    //     .entry(egui::FontFamily::Proportional)
    //     .or_default()
    //     .insert(0, "test_font".to_owned());

    // ---

    // #[cfg(target_os = "macos")]
    // if let Ok(font) = std::fs::read("/System/Library/Fonts/Hiragino Sans GB.ttc") {

    //     fonts.font_data.insert(
    //         "hiragino-sans-gb".to_owned(),
    //         // egui::FontData::from_static(include_bytes!("../../resources/fonts/Open Sans.ttf")),
    //         egui::FontData::from_owned(font),
    //     );

    //     fonts
    //         .families
    //         .entry(egui::FontFamily::Proportional)
    //         .or_default()
    //         // .insert(0, "hiragino".to_owned());
    //         .push("hiragino-sans-gb".to_owned());
    // }

    cc.egui_ctx.set_fonts(fonts);
}
