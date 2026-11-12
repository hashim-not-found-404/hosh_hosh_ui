use component_style_parts::draw_border;
use component_style_parts::draw_glitches;
use component_style_parts::draw_glow;
use component_style_parts::drow_random_horizontal_lines;
use component_style_parts::is_animation;
use component_style_parts::make_border;
use component_style_parts::make_glitches;
use component_style_parts::make_glow;
use component_style_parts::make_inner_rect;
use component_style_parts::make_random_horizontal_lines;
use prelude::InputField;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::Event;
use prelude::egui::FontId;
use prelude::egui::Key;
use prelude::egui::Painter;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::pos2;

const CORNER: f32 = 8.0;
const BORDER_WIDTH: f32 = 1.0;
const HOVER_SECS: f32 = 0.20;
const FOCUS_SECS: f32 = 0.20;
const GLITCH_DENSE: f32 = 400.0;
const PAD: f32 = 12.0;

const TEXT_ALPHA: f32 = 160.0;
const TEXT_ALPHA_FOCUSED: f32 = 230.0;
const PLACEHOLDER_ALPHA: f32 = 90.0;

pub struct DefaultInputField;

impl InputField for DefaultInputField {
    fn render(ui: &mut Ui, _t: &dyn Theme, value: &mut String, label: &str) -> Response {
        paint(ui, Color32::WHITE, value, label)
    }
}

pub fn paint(ui: &mut Ui, color_1: Color32, value: &mut String, label: &str) -> Response {
    let (outer_rect, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer_rect.width() <= 0.0 || outer_rect.height() <= 0.0 {
        return response;
    }

    let time = ui.input(|i| i.time);
    let seed = time.to_bits();
    let id = response.id;

    let was_focused = ui.ctx().memory(|m| m.has_focus(id));

    // ── Keyboard input (only when already focused) ───────────────
    if was_focused {
        let mut new_text = String::new();
        let mut backspace = 0;
        let mut blur = false;

        ui.input_mut(|i| {
            i.events.retain(|ev| match ev {
                Event::Text(t) => {
                    new_text.push_str(t);
                    false
                }
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    backspace += 1;
                    false
                }
                // Only Escape blurs. Enter is intentionally left in
                // the event stream so the caller can treat it as
                // "submit" without any visual animation from us.
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                } => {
                    blur = true;
                    false
                }
                _ => true,
            });
        });

        let mut changed = false;
        if !new_text.is_empty() {
            value.push_str(&new_text);
            changed = true;
        }
        for _ in 0..backspace {
            if value.pop().is_some() {
                changed = true;
            }
        }
        if changed {
            response.mark_changed();
        }
        if blur {
            ui.ctx().memory_mut(|m| m.surrender_focus(id));
        }
    }

    // ── Focus acquire / release ──────────────────────────────────
    if response.clicked() {
        ui.ctx().memory_mut(|m| m.request_focus(id));
    } else if was_focused && ui.input(|i| i.pointer.any_click()) {
        ui.ctx().memory_mut(|m| m.surrender_focus(id));
    }

    let focused = ui.ctx().memory(|m| m.has_focus(id));

    // ── Animations ───────────────────────────────────────────────
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("h"), response.hovered(), HOVER_SECS);
    let focus_t = ui
        .ctx()
        .animate_bool_with_time(id.with("f"), focused, FOCUS_SECS);
    let is_animate_hover = is_animation(ui, id.with("ah"), 0.16, response.hovered());
    let is_animate_click = is_animation(ui, id.with("ac"), 0.16, response.clicked());

    // ── Base layers ──────────────────────────────────────────────
    let inner_rect = make_inner_rect(&outer_rect);
    let painter = ui.painter();
    let points = make_border(&inner_rect, CORNER);

    draw_border(painter, color_1, points.clone(), BORDER_WIDTH);

    draw_content(
        painter,
        &inner_rect,
        color_1,
        value,
        label,
        focused,
        focus_t,
        time,
    );

    // ── Glow (continuous while hovered OR focused) ───────────────
    let activity = hover.max(focus_t);
    if activity > 0.001 {
        let glows = make_glow(&outer_rect, &points, (activity / 0.25) as u8);
        draw_glow(painter, color_1, &glows);
    }

    // ── Glitch + random lines (bursts on hover / click) ──────────
    if is_animate_hover || is_animate_click {
        let glitches = make_glitches(&points, GLITCH_DENSE, seed);
        let lines = make_random_horizontal_lines(&points, 25.0, seed);
        draw_glitches(painter, color_1, glitches);
        drow_random_horizontal_lines(painter, color_1, lines, 1.0);
    }

    // ── Interaction polish ───────────────────────────────────────
    if response.hovered() || focused {
        ui.ctx().set_cursor_icon(CursorIcon::Text);
    }
    if focused
        || is_animate_hover
        || is_animate_click
        || (0.001..0.999).contains(&hover)
        || (0.001..0.999).contains(&focus_t)
    {
        ui.ctx().request_repaint();
    }

    response
}

#[allow(clippy::too_many_arguments)]
fn draw_content(
    painter: &Painter,
    rect: &Rect,
    color: Color32,
    value: &str,
    label: &str,
    focused: bool,
    focus_t: f32,
    time: f64,
) {
    let font = FontId::monospace(16.0);
    let cy = rect.center().y;
    let x0 = rect.left() + PAD;

    // ── Empty: show label on the left as a hint ─────────────────
    if value.is_empty() {
        painter.text(
            pos2(x0, cy),
            Align2::LEFT_CENTER,
            label,
            font,
            with_alpha(color, PLACEHOLDER_ALPHA),
        );

        // Cursor sits at the very start while empty.
        if focused {
            let blink = ((time * 2.0).sin() * 0.5 + 0.5) as f32;
            if blink > 0.3 {
                painter.line_segment(
                    [pos2(x0, cy - 8.0), pos2(x0, cy + 8.0)],
                    Stroke::new(1.5, color),
                );
            }
        }
        return;
    }

    // ── Has value: label is gone, just show the text ────────────
    let alpha = TEXT_ALPHA + (TEXT_ALPHA_FOCUSED - TEXT_ALPHA) * focus_t;
    let text_color = with_alpha(color, alpha);

    let vg = painter.layout_no_wrap(value.to_string(), font.clone(), text_color);
    let value_w = vg.size().x;
    let value_h = vg.size().y;
    painter.galley(pos2(x0, cy - value_h * 0.5), vg, text_color);

    // Cursor after the value.
    if focused {
        let blink = ((time * 2.0).sin() * 0.5 + 0.5) as f32;
        if blink > 0.3 {
            let cx = x0 + value_w + 1.0;
            painter.line_segment(
                [pos2(cx, cy - 8.0), pos2(cx, cy + 8.0)],
                Stroke::new(1.5, color),
            );
        }
    }
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        alpha.clamp(0.0, 255.0) as u8,
    )
}
