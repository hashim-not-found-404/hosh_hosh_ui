pub use egui;
use egui::Color32;
use egui::CornerRadius;
use egui::Vec2;

pub trait Theme: Send + Sync + 'static {
    // ── Semantic colors ─────────────────────────────────────────────
    fn primary(&self) -> Color32;
    fn on_primary(&self) -> Color32;
    fn secondary(&self) -> Color32;
    fn danger(&self) -> Color32;
    fn success(&self) -> Color32;

    // ── Surfaces & text ─────────────────────────────────────────────
    fn background(&self) -> Color32;
    fn surface(&self) -> Color32;
    fn surface_alt(&self) -> Color32;
    fn text(&self) -> Color32;
    fn text_muted(&self) -> Color32;
    fn border(&self) -> Color32;

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
