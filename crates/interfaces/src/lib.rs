use egui::Color32;
use egui::CornerRadius;
use egui::Response;
use egui::Ui;
use egui::Vec2;

pub trait Theme: Send + Sync + 'static {
    // ── Semantic colors ─────────────────────────────────────────────
    fn primary(&self) -> Color32 {
        Color32::from_rgb(120, 120, 130)
    }
    fn on_primary(&self) -> Color32 {
        Color32::WHITE
    }
    fn secondary(&self) -> Color32 {
        Color32::from_rgb(60, 60, 70)
    }
    fn danger(&self) -> Color32 {
        Color32::from_rgb(180, 70, 70)
    }
    fn success(&self) -> Color32 {
        Color32::from_rgb(70, 160, 90)
    }

    // ── Surfaces & text ─────────────────────────────────────────────
    fn background(&self) -> Color32 {
        Color32::from_rgb(30, 30, 34)
    }
    fn surface(&self) -> Color32 {
        Color32::from_rgb(40, 40, 46)
    }
    fn surface_alt(&self) -> Color32 {
        Color32::from_rgb(50, 50, 58)
    }
    fn text(&self) -> Color32 {
        Color32::from_rgb(220, 220, 225)
    }
    fn text_muted(&self) -> Color32 {
        Color32::from_rgb(150, 150, 160)
    }
    fn border(&self) -> Color32 {
        Color32::from_rgb(70, 70, 80)
    }

    // ── Defaults — override only if you need to ─────────────────────
    fn radius(&self) -> CornerRadius {
        CornerRadius::same(8)
    }

    fn button_padding(&self) -> Vec2 {
        Vec2::new(14.0, 8.0)
    }

    fn spacing(&self) -> f32 {
        10.0
    }
}

pub struct DefaultTheme;

impl Theme for DefaultTheme {}

pub trait PrimaryButton: Send + Sync + 'static {
    fn render(ui: &mut Ui, theme: &dyn Theme, label: &str) -> Response;
}

pub trait SecondaryButton: Send + Sync + 'static {
    fn render(ui: &mut Ui, theme: &dyn Theme, label: &str) -> Response;
}

pub trait GhostButton: Send + Sync + 'static {
    fn render(ui: &mut Ui, theme: &dyn Theme, label: &str) -> Response;
}

pub trait DangerButton: Send + Sync + 'static {
    fn render(ui: &mut Ui, theme: &dyn Theme, label: &str) -> Response;
}
