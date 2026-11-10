use prelude::Label;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::FontId;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

// ─── Tuning ──────────────────────────────────────────────────────────
/// Left inset from the allocated cell to the accent bar.
const BAR_PAD: f32 = 3.0;
/// Accent bar width, as a fraction of the computed font size.
const BAR_WIDTH_RATIO: f32 = 0.10;
/// Accent bar height, as a fraction of the computed font size.
const BAR_HEIGHT_RATIO: f32 = 0.95;
/// Diagonal tip length at the top of the accent bar, as a fraction of
/// the bar width. A value of 0 turns the bar into a plain rectangle.
const BAR_TIP_RATIO: f32 = 1.6;

// ─── Public API ──────────────────────────────────────────────────────
pub struct DefaultLabel;

impl Label for DefaultLabel {
    fn render(ui: &mut Ui, t: &dyn Theme, text: &str) -> Response {
        paint(ui, text, t.text(), t.brand())
    }
}

/// Cyberpunk label — a leading accent bar with a diagonal tip, then
/// the text. Fills `ui.available_size()`, auto-sizes the font to fit
/// both the row height and the text's length.
///
/// Colors injected by the caller:
///   * `text_color` — the label text
///   * `accent`     — the bar (and its tip)
pub fn paint(ui: &mut Ui, text: &str, text_color: Color32, accent: Color32) -> Response {
    let (outer, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    if outer.width() <= 0.0 || outer.height() <= 0.0 || text.is_empty() {
        return response;
    }

    let painter = ui.painter().with_clip_rect(outer);

    // ── Font sizing ──────────────────────────────────────────────
    // By height first, then by width so long labels shrink instead of
    // clipping. Minimum 6pt so even tiny labels stay legible.
    let font_by_height = (outer.height() * 0.55).clamp(8.0, 24.0);
    let bar_w = (font_by_height * BAR_WIDTH_RATIO).max(2.0);
    let spacing = font_by_height * 0.45;
    let avail_text_w = (outer.width() - BAR_PAD - bar_w - spacing).max(0.0);
    let char_count = text.chars().count().max(1) as f32;
    let font_by_width = avail_text_w / char_count / 0.6;
    let font_size = font_by_height.min(font_by_width).max(6.0);

    let cy = outer.center().y;
    let bar_x = outer.left() + BAR_PAD;
    let bar_h = font_size * BAR_HEIGHT_RATIO;

    // ── Accent bar with a diagonal tip at the top ───────────────
    let tip_len = bar_w * BAR_TIP_RATIO;
    let top = cy - bar_h / 2.0;
    let bottom = cy + bar_h / 2.0;

    if tip_len > 0.5 {
        painter.add(prelude::egui::Shape::convex_polygon(
            vec![
                pos2(bar_x, bottom),
                pos2(bar_x, top + tip_len * 0.5),
                pos2(bar_x + tip_len * 0.5, top),
                pos2(bar_x + bar_w, top + tip_len * 0.5),
                pos2(bar_x + bar_w, bottom),
            ],
            accent,
            Stroke::NONE,
        ));
    } else {
        painter.rect_filled(
            Rect::from_min_size(pos2(bar_x, top), vec2(bar_w, bar_h)),
            0.0,
            accent,
        );
    }

    // ── Text ────────────────────────────────────────────────────
    let font = FontId::monospace(font_size);
    painter.text(
        pos2(bar_x + bar_w + spacing, cy),
        Align2::LEFT_CENTER,
        text,
        font,
        text_color,
    );

    response
}
