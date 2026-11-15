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
use prelude::State;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
use prelude::egui::Key;
use prelude::egui::Painter;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::update;

const CORNER: f32 = 8.0;
const BORDER_WIDTH: f32 = 1.0;
const HOVER_SECS: f32 = 0.20;
const FOCUS_SECS: f32 = 0.20;
const GLITCH_DENSE: f32 = 400.0;
const PAD: f32 = 12.0;
const FONT_SIZE: f32 = 16.0;

const TEXT_ALPHA: f32 = 160.0;
const TEXT_ALPHA_FOCUSED: f32 = 230.0;
const PLACEHOLDER_ALPHA: f32 = 90.0;
const SELECTION_ALPHA: f32 = 80.0;

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
    let state_id = id.with("text_edit");

    // Monospace char width — used for both click-to-position and painting.
    let inner_rect = make_inner_rect(&outer_rect);
    let font = FontId::monospace(FONT_SIZE);
    let char_w = {
        let p = ui.painter();
        p.layout_no_wrap("M".to_string(), font.clone(), Color32::WHITE)
            .size()
            .x
            .max(0.001)
    };
    let text_x0 = inner_rect.left() + PAD;

    let was_focused = ui.ctx().memory(|m| m.has_focus(id));

    // ── Focus management ────────────────────────────────────────
    let clicked_any = response.clicked() || response.double_clicked() || response.triple_clicked();

    if clicked_any {
        ui.ctx().memory_mut(|m| m.request_focus(id));
    } else if was_focused && ui.input(|i| i.pointer.any_click()) {
        ui.ctx().memory_mut(|m| m.surrender_focus(id));
    }

    let focused = ui.ctx().memory(|m| m.has_focus(id));

    // ── Text editing state ──────────────────────────────────────
    // Persisted across frames. On blur we drop it, so the next
    // focus starts with a fresh state.
    let mut state: State = ui
        .ctx()
        .data(|d| d.get_temp::<State>(state_id))
        .unwrap_or_else(|| State::from(value.as_str()));

    // External changes: the caller may have set `value` from
    // somewhere else. If the state doesn't match, resync.
    if state.get_text().as_str() != value.as_str() {
        state = State::from(value.as_str());
    }

    if focused {
        // Handle click → cursor / word / select-all.
        if clicked_any {
            if let Some(pos) = response.interact_pointer_pos() {
                let local_x = (pos.x - text_x0).max(0.0);
                let char_at = (local_x / char_w).round() as usize;

                if response.triple_clicked() {
                    state.select_all();
                } else if response.double_clicked() {
                    state.select_word_at(char_at);
                } else {
                    state.set_cursor_char(char_at);
                }
            }
        }

        // Feed keyboard / text / paste events through the editor.
        update(ui, &mut state);

        // Sync back to the caller's string if it changed.
        if state.get_text().as_str() != value.as_str() {
            *value = state.get_text().to_string();
            response.mark_changed();
        }

        // Escape blurs. `text_edit::update` leaves Escape in the
        // event stream (it's not a text-editing event), so we can
        // check it here directly.
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            ui.ctx().memory_mut(|m| m.surrender_focus(id));
        }
    }

    // Persist for next frame — only while focused. On blur we drop
    // it, so the next focus starts fresh.
    if focused {
        ui.ctx()
            .data_mut(|d| d.insert_temp(state_id, state.clone()));
    } else {
        ui.ctx().data_mut(|d| d.remove::<State>(state_id));
    }

    // ── Animations ──────────────────────────────────────────────
    let hover = ui
        .ctx()
        .animate_bool_with_time(id.with("h"), response.hovered(), HOVER_SECS);
    let focus_t = ui
        .ctx()
        .animate_bool_with_time(id.with("f"), focused, FOCUS_SECS);
    let is_animate_hover = is_animation(ui, id.with("ah"), 0.16, response.hovered());
    let is_animate_click = is_animation(ui, id.with("ac"), 0.16, response.clicked());

    // ── Base layers ─────────────────────────────────────────────
    let painter = ui.painter();
    let points = make_border(&inner_rect, CORNER);

    draw_border(painter, color_1, points.clone(), BORDER_WIDTH);

    draw_content(
        painter,
        &inner_rect,
        color_1,
        &state,
        label,
        focused,
        focus_t,
        time,
        char_w,
    );

    // ── Glow (continuous while hovered OR focused) ──────────────
    let activity = hover.max(focus_t);
    if activity > 0.001 {
        let glows = make_glow(&outer_rect, &points, (activity / 0.25) as u8);
        draw_glow(painter, color_1, &glows);
    }

    // ── Glitch + random lines (bursts on hover / click) ─────────
    if is_animate_hover || is_animate_click {
        let glitches = make_glitches(&points, GLITCH_DENSE, seed);
        let lines = make_random_horizontal_lines(&points, 25.0, seed);
        draw_glitches(painter, color_1, glitches);
        drow_random_horizontal_lines(painter, color_1, lines, 1.0);
    }

    // ── Interaction polish ──────────────────────────────────────
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

// ─── Paint ─────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_content(
    painter: &Painter,
    rect: &Rect,
    color: Color32,
    state: &State,
    label: &str,
    focused: bool,
    focus_t: f32,
    time: f64,
    char_w: f32,
) {
    let font = FontId::monospace(FONT_SIZE);
    let cy = rect.center().y;
    let x0 = rect.left() + PAD;

    // ── Empty: show label on the left as a hint ─────────────────
    if state.get_text().is_empty() {
        painter.text(
            pos2(x0, cy),
            Align2::LEFT_CENTER,
            label,
            font,
            with_alpha(color, PLACEHOLDER_ALPHA),
        );

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

    // ── Has value: selection highlight + text + cursor ─────────
    let alpha = TEXT_ALPHA + (TEXT_ALPHA_FOCUSED - TEXT_ALPHA) * focus_t;
    let text_color = with_alpha(color, alpha);

    let vg = painter.layout_no_wrap(state.get_text().to_string(), font.clone(), text_color);
    let value_h = vg.size().y;
    let text_y = cy - value_h * 0.5;

    // Selection highlight behind the text.
    if let Some(chars) = state.selection_char_range() {
        let x_start = x0 + chars.start as f32 * char_w;
        let x_end = x0 + chars.end as f32 * char_w;
        painter.rect_filled(
            Rect::from_min_max(pos2(x_start, text_y), pos2(x_end, text_y + value_h)),
            0.0,
            with_alpha(color, SELECTION_ALPHA),
        );
    }

    painter.galley(pos2(x0, text_y), vg, text_color);

    // Cursor at edit position.
    if focused {
        let blink = ((time * 2.0).sin() * 0.5 + 0.5) as f32;
        if blink > 0.3 {
            let x = x0 + state.cursor_char() as f32 * char_w;
            painter.line_segment(
                [pos2(x, cy - 8.0), pos2(x, cy + 8.0)],
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
