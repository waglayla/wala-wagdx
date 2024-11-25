use super::*;
use crate::components::*;

pub struct Outline {
    selected_tab: Tab,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumIter)]
pub enum Tab {
    Test,
    Wallet,
    NetworkInfo,
    WalaNode,
    About,
}

impl Tab {
    fn label(&self) -> &'static str {
        match self {
            Tab::Test => i18n("Hello!"),
            Tab::Wallet => i18n("Wallet"),
            Tab::NetworkInfo => i18n("Network Info"),
            Tab::WalaNode => i18n("WALA Node"),
            Tab::About => i18n("About"),
        }
    }

    fn component(&self) -> TypeId {
        match self {
            Tab::Test => TypeId::of::<hello::Hello>(),
            Tab::Wallet => TypeId::of::<blank::Blank>(),
            Tab::NetworkInfo => TypeId::of::<blank::Blank>(),
            Tab::WalaNode => TypeId::of::<console::DaemonConsole>(),
            Tab::About => TypeId::of::<blank::Blank>(),
        }
    }
}

impl Default for Outline {
    fn default() -> Self {
        Self {
            selected_tab: Tab::iter().next().unwrap(),
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
        let panel_fill = ctx.style().visuals.panel_fill;
        let darkened_fill = panel_fill.linear_multiply_rgb(0.5);

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

                // Set the text style for buttons
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::new(30.0, get_font_family("DINishCondensed", false, false))
                );

                ui.add_space(132.);

                for tab in Tab::iter() {
                    if self.tab_button(ui, ctx, tab) {
                        self.selected_tab = tab;
                        // You might want to notify the core about tab changes
                        core.set_active_component_by_type(tab.component().clone());
                    }
                }
            });
    }
}

impl Outline {
    fn tab_button(&self, ui: &mut Ui, ctx: &Context, tab: Tab) -> bool {
        let panel_fill = ctx.style().visuals.panel_fill;
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
    
        if ui.is_rect_visible(rect) {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            
            ui.painter().rect_filled(
                rect,
                Rounding::ZERO,
                bg_color,
            );
    
            let text_padding = 12.0;
            let text_rect = rect.shrink2(vec2(text_padding, 0.0));
            
            let text_color = if selected {
                Color32::WHITE
            } else if response.hovered() {
                Color32::LIGHT_GRAY
            } else {
                ctx.style().visuals.widgets.inactive.text_color().linear_multiply_rgb(0.66)
            };

            ui.painter().text(
                text_rect.left_top(),
                egui::Align2::LEFT_TOP,
                tab.label(),
                ui.style().text_styles[&egui::TextStyle::Button].clone(),
                text_color,
            );
        }
    
        response.clicked()
    }
}