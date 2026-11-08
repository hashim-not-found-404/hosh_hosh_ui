pub use egui;
use egui::Color32;
use egui::Context;
use egui::Id;
use egui::Stroke;
use egui::Theme as EguiTheme;
use egui::ThemePreference;
pub use interfaces;
pub use interfaces::Theme;
use std::sync::Arc;

const THEME_KEY: &str = "hosh_hosh_ui::theme";

/// Store the theme in `egui::Context` and push it into egui's visuals.
pub fn set_theme(ctx: &Context, theme: impl Theme) {
    ctx.data_mut(|d| {
        d.insert_temp(Id::unique(THEME_KEY), Arc::new(theme) as Arc<dyn Theme>);
    });
    apply_to_context(ctx);
    ctx.request_repaint();
}

/// Read the current theme, falling back to `FallbackTheme`.
pub fn theme(ctx: &Context) -> Arc<dyn Theme> {
    ctx.data(|d| d.get_temp::<Arc<dyn Theme>>(Id::unique(THEME_KEY)))
        .unwrap_or_else(|| Arc::new(FallbackTheme) as Arc<dyn Theme>)
}

/// Apply the current theme's tokens to egui's global `Visuals`.
///
/// Widgets we ship use tokens directly, so this only matters for
/// "bare" egui widgets (text edits, panels, separators, …) that end
/// users mix in alongside ours.
pub fn apply_to_context(ctx: &Context) {
    ctx.options_mut(|opt| {
        opt.theme_preference = ThemePreference::Dark;
    });

    let t = theme(ctx);
    let mut style = (*ctx.style_of(EguiTheme::Dark)).clone();

    {
        let v = &mut style.visuals;
        v.dark_mode = true;
        v.panel_fill = t.background();
        v.window_fill = t.surface();
        v.extreme_bg_color = t.surface_alt();
        v.faint_bg_color = t.surface_alt();
        v.override_text_color = Some(t.text());
        v.selection.bg_fill = t.primary();
        v.selection.stroke = Stroke::new(1.0, t.primary());
        v.hyperlink_color = t.primary();

        let r = t.radius();
        v.widgets.noninteractive.corner_radius = r;
        v.widgets.inactive.corner_radius = r;
        v.widgets.hovered.corner_radius = r;
        v.widgets.active.corner_radius = r;

        v.widgets.noninteractive.bg_fill = t.surface();
        v.widgets.noninteractive.weak_bg_fill = t.surface();
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, t.border());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, t.text());

        v.widgets.inactive.bg_fill = t.surface_alt();
        v.widgets.inactive.weak_bg_fill = t.surface_alt();
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, t.border());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, t.text());

        v.widgets.hovered.bg_fill = t.border();
        v.widgets.hovered.weak_bg_fill = t.border();
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.primary());
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, t.text());

        v.widgets.active.bg_fill = t.primary();
        v.widgets.active.weak_bg_fill = t.primary();
        v.widgets.active.bg_stroke = Stroke::new(1.0, t.primary());
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::BLACK);
    }

    style.spacing.item_spacing = egui::vec2(t.spacing(), t.spacing());
    style.spacing.button_padding = t.button_padding();
    style.spacing.interact_size.y = 32.0;

    ctx.set_style_of(EguiTheme::Dark, style);
}

/// Intentionally plain. Only used if the app never calls `set_theme`.
///
/// Users should always install a real theme (e.g. `theme_default::DefaultTheme`).
pub struct FallbackTheme;

impl Theme for FallbackTheme {
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
}
