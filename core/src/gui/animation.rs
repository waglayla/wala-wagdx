use egui::{Context, Color32, Id};

pub trait ColorAnimation  {
    fn animate_color_with_time(&self, id: Id, target: Color32, animation_time: f32) -> Color32;
}

impl ColorAnimation for Context {
    fn animate_color_with_time(&self, id: Id, target: Color32, animation_time: f32) -> Color32 {
        let current = self.data(|data| {
            data.get_temp::<Color32>(id)
                .unwrap_or(target)
        });

        // Calculate individual factors for R, G, and B
        let r_factor = self.animate_value_with_time(id.with("r"), target.r() as f32, animation_time);
        let g_factor = self.animate_value_with_time(id.with("g"), target.g() as f32, animation_time);
        let b_factor = self.animate_value_with_time(id.with("b"), target.b() as f32, animation_time);

        let animated_color = Color32::from_rgb(
            r_factor as u8,
            g_factor as u8,
            b_factor as u8,
        );

        self.data_mut(|data| {
            data.insert_temp(id, animated_color);
        });

        animated_color
    }
}
