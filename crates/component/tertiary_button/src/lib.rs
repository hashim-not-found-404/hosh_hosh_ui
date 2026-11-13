use component_style_parts::draw_border;
use component_style_parts::draw_glitches;
use component_style_parts::draw_glow;
use component_style_parts::draw_text;
use component_style_parts::drow_random_horizontal_lines;
use component_style_parts::is_animation;
use component_style_parts::make_border;
use component_style_parts::make_glitches;
use component_style_parts::make_glow;
use component_style_parts::make_inner_rect;
use component_style_parts::make_random_horizontal_lines;
use component_style_parts::make_text;
use prelude::TertiaryButton;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Ui;

const CORNER: f32 = 8.0;
const HOVER_SECS: f32 = 0.20;
const BORDER_WIDTH: f32 = 1.0;
const TEXT_DRIFT: f32 = 2.5;
const GLITCH_DENSE: f32 = 400.0;

pub struct DefaultTertiaryButton;

impl TertiaryButton for DefaultTertiaryButton {
    fn render(ui: &mut Ui, theme: &dyn Theme, label: &str) -> Response {
        paint(ui, Color32::WHITE, label)
    }
}

fn paint(ui: &mut Ui, color_1: Color32, label: &str) -> Response {
    let (outer_rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer_rect.width() <= 0.0 || outer_rect.height() <= 0.0 {
        return response;
    }

    let time = ui.input(|i| i.time);
    let seed = time.to_bits();

    // Two independent bursts: one fires on hover-enter, one fires on
    // click. Each is a one-shot 0.16 s window — same timing as the
    // primary / secondary buttons.
    let is_animate_hover = is_animation(
        ui,
        response.id.with("button-hover"),
        0.16,
        response.hovered(),
    );
    let is_animate_click = is_animation(
        ui,
        response.id.with("button-click"),
        0.16,
        response.clicked(),
    );
    let animate = is_animate_hover || is_animate_click;

    // Smooth 0→1 fade for the border. Independent of the burst — the
    // border stays at full alpha for as long as the pointer is over
    // the button, whereas the glitch/ghost effect is only 0.16 s.
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("h"), response.hovered(), HOVER_SECS);

    let inner_rect = make_inner_rect(&outer_rect);
    let points = make_border(&inner_rect, CORNER);
    let painter = ui.painter();

    // ── Border — white, fades in on hover ────────────────────────
    if hover > 0.01 {
        draw_border(painter, color_1, points.clone(), BORDER_WIDTH);
    }

    // ── Glow — white, while hovered ──────────────────────────────
    if hover > 0.001 {
        let glows = make_glow(&outer_rect, &points, (hover / 0.25) as u8);
        draw_glow(painter, color_1, &glows);
    }

    // ── Plain text — white, always visible ───────────────────────
    painter.text(
        inner_rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::monospace(16.0),
        color_1,
    );

    // ── Burst — glitches, lines, chromatic text ghosts ───────────
    if animate {
        // `make_text` produces the plain text plus three chromatic
        // ghosts drifting around it, driven by `make_text`'s own RNG.
        // `draw_text` reverses the iteration so the solid text lands
        // on top of the ghosts — the "chromatic fringe" effect.
        let texts = make_text(
            &points,
            label,
            Align2::CENTER_CENTER,
            color_1,
            time.fract() as f32,
            TEXT_DRIFT,
            seed,
        );
        draw_text(
            painter,
            Align2::CENTER_CENTER,
            texts,
            FontId::monospace(16.0),
        );

        let glitches = make_glitches(&points, GLITCH_DENSE, seed);
        let lines = make_random_horizontal_lines(&points, 25.0, seed);
        draw_glitches(painter, color_1, glitches);
        drow_random_horizontal_lines(painter, color_1, lines, 1.0);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    ui.request_repaint();

    response
}
