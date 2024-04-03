use std::f32::consts::TAU;

use egui::{Color32, FontId, Stroke, Ui, Vec2};

pub struct CircleDisplay {
    stroke_width: f32,
    stroke_color: Color32,
    font_size: f32,
    segments: i32,
    size: f32,
    renderbackground: bool,
}

impl CircleDisplay {
    pub fn new() -> CircleDisplay {
        CircleDisplay::default()
    }
    pub fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }
    pub fn stroke_color(mut self, color: Color32) -> Self {
        self.stroke_color = color;
        self
    }
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    pub fn segmets(mut self, amount: i32) -> Self {
        self.segments = amount;
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    pub fn render_background(mut self) -> Self {
        self.renderbackground = true;
        self
    }
    pub fn build<'a>(self, value: &'a i32, max_value: &'a i32) -> impl egui::Widget + 'a {
        move |ui: &mut Ui| self.render(ui, value, max_value)
    }
}

impl CircleDisplay {
    fn render(self, ui: &mut egui::Ui, value: &i32, max_value: &i32) -> egui::Response {
        let stroke = Stroke {
            width: self.stroke_width,
            color: self.stroke_color,
        };
        let desired_size = ui.spacing().interact_size.y * egui::Vec2::splat(self.size);
        let (rect, response) =
            ui.allocate_exact_size(desired_size, egui::Sense::focusable_noninteractive());
        let normalized = *value as f32 / *max_value as f32;
        if ui.is_rect_visible(rect) {
            let visuals = ui.style().noninteractive();
            let rect = rect.expand(visuals.expansion);
            let radius = 0.5 * rect.height() * 0.9;
            if self.renderbackground {
                let mut s = visuals.bg_stroke.to_owned();
                s.width = self.stroke_width;
                ui.painter().rect(rect, radius, visuals.bg_fill, s);
            }
            let part = TAU / self.segments as f32;
            let center = rect.center();
            for i in 0..self.segments {
                let points = [
                    center + Vec2::angled((i as f32 * part * normalized) - TAU / 4.) * radius,
                    center
                        + Vec2::angled(
                            (((i as f32 + 1f32) * part * normalized) - TAU / 4.)
                                + self.stroke_width * 0.005,
                        ) * radius,
                ];
                ui.painter().line_segment(points, stroke);
            }
            let text = ui.painter().layout_no_wrap(
                format!(" {}\n/{}", limit_number(value), limit_number(max_value)),
                FontId::monospace(self.font_size),
                visuals.text_color(),
            );
            let t_center = text.size() / 2.;
            ui.painter()
                .galley(center - t_center, text, visuals.text_color());
        }

        response
    }
}
fn limit_number(num: &i32) -> String {
    if num >= &1000 {
        let all = num.to_string();
        let str = &all[..all.len() - 3];
        format!("{str}K")
    } else {
        num.to_string()
    }
}

impl Default for CircleDisplay {
    fn default() -> Self {
        Self {
            stroke_width: 1.0,
            stroke_color: Color32::RED,
            // 14 grabed  from the default implementation
            font_size: 14.0,
            segments: 20,
            size: 1.0,
            renderbackground: false,
        }
    }
}
