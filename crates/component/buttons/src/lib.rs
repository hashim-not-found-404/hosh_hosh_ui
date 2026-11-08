use prelude::egui::Button;
use prelude::egui::Color32;
use prelude::egui::Response;
use prelude::egui::RichText;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::vec2;
use prelude::theme;

pub trait ButtonExt {
    fn primary_button(&mut self, label: &str) -> Response;
    fn secondary_button(&mut self, label: &str) -> Response;
    fn ghost_button(&mut self, label: &str) -> Response;
    fn danger_button(&mut self, label: &str) -> Response;
}

impl ButtonExt for Ui {
    fn primary_button(&mut self, label: &str) -> Response {
        let t = theme(self.ctx());
        let bg = t.primary();
        let text = t.on_primary();
        let hover = lighten(bg, 14);
        let radius = t.radius();
        paint_states(self, bg, hover, hover, bg);
        self.add(
            Button::new(RichText::new(label).color(text).strong())
                .stroke(Stroke::new(1.0, bg))
                .corner_radius(radius)
                .min_size(vec2(0.0, 32.0)),
        )
    }

    fn secondary_button(&mut self, label: &str) -> Response {
        let t = theme(self.ctx());
        let bg = t.secondary();
        let hover = t.border();
        let radius = t.radius();
        paint_states(self, bg, hover, hover, t.border());
        self.add(
            Button::new(RichText::new(label).color(t.text()).strong())
                .stroke(Stroke::new(1.0, t.border()))
                .corner_radius(radius)
                .min_size(vec2(0.0, 32.0)),
        )
    }

    fn ghost_button(&mut self, label: &str) -> Response {
        let t = theme(self.ctx());
        let hover = t.secondary();
        let radius = t.radius();
        paint_states(
            self,
            Color32::TRANSPARENT,
            hover,
            hover,
            Color32::TRANSPARENT,
        );
        self.add(
            Button::new(RichText::new(label).color(t.text_muted()))
                .stroke(Stroke::NONE)
                .corner_radius(radius)
                .min_size(vec2(0.0, 32.0)),
        )
    }

    fn danger_button(&mut self, label: &str) -> Response {
        let t = theme(self.ctx());
        let bg = t.danger();
        let hover = lighten(bg, 14);
        let radius = t.radius();
        paint_states(self, bg, hover, hover, bg);
        self.add(
            Button::new(RichText::new(label).color(Color32::WHITE).strong())
                .stroke(Stroke::new(1.0, bg))
                .corner_radius(radius)
                .min_size(vec2(0.0, 32.0)),
        )
    }
}

fn paint_states(ui: &mut Ui, fill: Color32, hover: Color32, active: Color32, stroke: Color32) {
    let v = &mut ui.style_mut().visuals.widgets;
    v.inactive.weak_bg_fill = fill;
    v.inactive.bg_fill = fill;
    v.inactive.bg_stroke = Stroke::new(1.0, stroke);
    v.hovered.weak_bg_fill = hover;
    v.hovered.bg_fill = hover;
    v.hovered.bg_stroke = Stroke::new(1.0, stroke);
    v.active.weak_bg_fill = active;
    v.active.bg_fill = active;
    v.active.bg_stroke = Stroke::new(1.0, stroke);
}

fn lighten(c: Color32, amt: u8) -> Color32 {
    Color32::from_rgb(
        c.r().saturating_add(amt),
        c.g().saturating_add(amt),
        c.b().saturating_add(amt),
    )
}
