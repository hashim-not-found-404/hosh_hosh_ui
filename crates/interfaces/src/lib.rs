use egui::Color32;
use egui::Response;
use egui::Ui;

/// A theme is a **palette of tokens**, not a mapping to component roles.
///
/// Each method answers one question: *"what color does this represent?"*
/// It does **not** answer *"which component should use this?"* — that
/// decision belongs to the caller (a button impl, a layout, the app).
///
/// For example, [`brand`][Self::brand] is *the theme's primary brand
/// accent*, not *the primary button's color*. A primary button happens
/// to use it for both its face and its border, but that's a choice made
/// in the button crate — not a semantic baked into the trait.
///
/// ## Accent containers and their "on" partners
///
/// Every accent token that is intended to host content (text, icons)
/// has a paired `on_X()` token that describes content placed *on* it.
/// This keeps contrast decisions on the theme side rather than spread
/// across every component:
///
/// | Container      | Content placed on it |
/// |----------------|----------------------|
/// | `brand()`      | `on_brand()`         |
/// | `brand_alt()`  | `on_brand_alt()`     |
/// | `danger()`     | `on_danger()`        |
/// | `success()`    | `on_success()`       |
///
/// ## What does **not** belong here
///
/// Metrics (radius, padding, spacing, font sizes) describe *shape and
/// layout*, not color. Putting them on `Theme` forces every theme to
/// answer "what is the button padding?" — a question unrelated to its
/// palette. Component metrics belong on the component (as constants) or
/// on a separate `Metrics` trait.
pub trait Theme: Send + Sync + 'static {
    // ── Brand accents ───────────────────────────────────────────────

    /// The theme's primary brand accent.
    fn brand(&self) -> Color32 {
        Color32::from_rgb(120, 120, 130)
    }

    /// Content color intended for use on top of [`brand`][Self::brand].
    fn on_brand(&self) -> Color32 {
        Color32::WHITE
    }

    /// The theme's secondary brand accent.
    fn brand_alt(&self) -> Color32 {
        Color32::from_rgb(60, 60, 70)
    }

    /// Content color intended for use on top of [`brand_alt`][Self::brand_alt].
    fn on_brand_alt(&self) -> Color32 {
        Color32::WHITE
    }

    // ── Semantic accents ────────────────────────────────────────────

    /// Destructive / error accent.
    fn danger(&self) -> Color32 {
        Color32::from_rgb(180, 70, 70)
    }

    /// Content color intended for use on top of [`danger`][Self::danger].
    fn on_danger(&self) -> Color32 {
        Color32::WHITE
    }

    /// Positive / confirmation accent.
    fn success(&self) -> Color32 {
        Color32::from_rgb(70, 160, 90)
    }

    /// Content color intended for use on top of [`success`][Self::success].
    fn on_success(&self) -> Color32 {
        Color32::WHITE
    }

    // ── Surfaces ────────────────────────────────────────────────────

    /// The root backdrop everything sits on.
    fn background(&self) -> Color32 {
        Color32::from_rgb(30, 30, 34)
    }

    /// A resting surface — a panel or card on top of [`background`][Self::background].
    fn surface(&self) -> Color32 {
        Color32::from_rgb(40, 40, 46)
    }

    /// A surface raised above [`surface`][Self::surface] — a nested card,
    /// a button face, a popover.
    fn surface_elevated(&self) -> Color32 {
        Color32::from_rgb(50, 50, 58)
    }

    // ── Foreground ──────────────────────────────────────────────────

    /// Default body text, intended for use on `background` / `surface`.
    fn text(&self) -> Color32 {
        Color32::from_rgb(220, 220, 225)
    }

    /// De-emphasized text — labels, captions, hints.
    fn text_muted(&self) -> Color32 {
        Color32::from_rgb(150, 150, 160)
    }

    /// Hairline / outline color, intended for use on `background` / `surface`.
    fn border(&self) -> Color32 {
        Color32::from_rgb(70, 70, 80)
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
