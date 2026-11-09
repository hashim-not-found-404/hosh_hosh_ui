use prelude::PrimaryButton;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CornerRadius;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Stroke;
use prelude::egui::StrokeKind;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// How far the glow extends beyond the button face. Reserved in the
/// footprint so the glow never bleeds into siblings.
const GLOW: f32 = 9.0;

pub struct DefaultPrimaryButton;

impl PrimaryButton for DefaultPrimaryButton {
    fn render(ui: &mut Ui, t: &dyn Theme, label: &str) -> Response {
        button(ui, t, label)
    }
}

fn button(ui: &mut Ui, t: &dyn Theme, label: &str) -> Response {
    // ── Footprint = face + glow margin on every side. ──────────────
    // The parent decides `face_w` by choosing the layout:
    //   vertical            → panel width
    //   horizontal_wrapped  → whatever fits on the current line
    //   allocate_ui_with…   → exactly what the app set
    let desired = ui.available_size();

    // Reserve the WHOLE footprint (face + glow), not just the face.
    let (outer, response) = ui.allocate_exact_size(desired, Sense::click());

    // The face is inset by GLOW on every side.
    let rect = outer.shrink(GLOW);

    // Clip every paint call to `outer`. Glow can reach `outer` but
    // never past it, so two side-by-side buttons can't overlap.
    let painter = ui.painter().with_clip_rect(outer);

    // ── Animations ─────────────────────────────────────────────────
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("cyber_hover"), response.hovered(), 0.16);

    let press = ui.ctx().animate_bool_with_time(
        response.id.with("cyber_press"),
        response.is_pointer_button_down_on(),
        0.05,
    );

    let primary = t.primary();
    let on_primary = t.on_primary();
    let corner_radius = CornerRadius::same(0);

    // 1. Outer glow — reaches exactly `outer`, no further.
    let glow = 0.35 + 0.65 * hover;
    for (expand, mult) in [(GLOW, 0.10), (GLOW * 0.6, 0.18), (GLOW * 0.25, 0.30)] {
        let alpha = (mult * glow * 255.0) as u8;
        painter.rect_filled(
            rect.expand(expand),
            corner_radius,
            Color32::from_rgba_unmultiplied(primary.r(), primary.g(), primary.b(), alpha),
        );
    }

    // 2. Dark base fill.
    painter.rect_filled(rect, corner_radius, t.background());

    // 3. Inner tint (hover only).
    if hover > 0.0 {
        painter.rect_filled(
            rect.shrink(1.0),
            corner_radius,
            Color32::from_rgba_unmultiplied(
                primary.r(),
                primary.g(),
                primary.b(),
                (32.0 * hover) as u8,
            ),
        );
    }

    // 4. Neon border.
    let border_alpha = (90.0 + 165.0 * hover) as u8;
    painter.rect_stroke(
        rect,
        corner_radius,
        Stroke::new(
            1.0 + 0.8 * hover,
            Color32::from_rgba_unmultiplied(primary.r(), primary.g(), primary.b(), border_alpha),
        ),
        StrokeKind::Inside,
    );

    // 5. Corner brackets.
    let blen = 10.0;
    let bracket = Stroke::new(2.0 + 0.8 * hover, primary);
    for (origin, dx, dy) in [
        (rect.left_top(), vec2(blen, 0.0), vec2(0.0, blen)),
        (rect.right_top(), vec2(-blen, 0.0), vec2(0.0, blen)),
        (rect.left_bottom(), vec2(blen, 0.0), vec2(0.0, -blen)),
        (rect.right_bottom(), vec2(-blen, 0.0), vec2(0.0, -blen)),
    ] {
        painter.line_segment([origin, origin + dx], bracket);
        painter.line_segment([origin, origin + dy], bracket);
    }

    // 6. Corner dots.
    for pt in [
        rect.left_top(),
        rect.right_top(),
        rect.left_bottom(),
        rect.right_bottom(),
    ] {
        painter.circle_filled(pt, 2.0, primary);
    }

    // 7. Scanline.
    if hover > 0.0 {
        let time = ui.input(|i| i.time) as f32;
        let y = rect.top() + (time * 80.0).rem_euclid(rect.height());
        painter.line_segment(
            [pos2(rect.left() + 3.0, y), pos2(rect.right() - 3.0, y)],
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(255, 255, 255, (100.0 * hover) as u8),
            ),
        );
        ui.ctx().request_repaint();
    }

    // 8. Glowing label.
    let text_color = lerp(primary, on_primary, hover.max(press));
    let font = FontId::monospace(15.0);
    if hover > 0.0 {
        let glow_color = Color32::from_rgba_unmultiplied(
            primary.r(),
            primary.g(),
            primary.b(),
            (140.0 * hover) as u8,
        );
        let d = 1.5;
        for off in [vec2(d, 0.0), vec2(-d, 0.0), vec2(0.0, d), vec2(0.0, -d)] {
            painter.text(
                rect.center() + off,
                Align2::CENTER_CENTER,
                label,
                font.clone(),
                glow_color,
            );
        }
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        text_color,
    );

    // 9. Press flash.
    if press > 0.0 {
        painter.rect_filled(
            rect,
            corner_radius,
            Color32::from_rgba_unmultiplied(255, 255, 255, (60.0 * press) as u8),
        );
    }

    // 10. Cursor.
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }

    // 11. Keep animating while a transition is in flight.
    if (0.001..0.999).contains(&hover) || (0.001..0.999).contains(&press) {
        ui.ctx().request_repaint();
    }

    response
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}
