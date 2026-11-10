use prelude::egui::pos2;
use prelude::egui::vec2;
use prelude::egui::Align2;
use prelude::egui::Area;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::Event;
use prelude::egui::FontId;
use prelude::egui::Key;
use prelude::egui::Order;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Shape;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::Vec2;
use prelude::InputList;
use prelude::Theme;

// ─── Tuning ──────────────────────────────────────────────────────────
const FACE_PAD: f32 = 6.0;
const CORNER_CUT: f32 = 8.0;
const GLITCH_DURATION: f64 = 0.22;
const HOVER_SECS: f32 = 0.16;
const PRESS_SECS: f32 = 0.05;
const FOCUS_SECS: f32 = 0.22;
const FLOAT_SECS: f32 = 0.15;
const INTRO_DURATION: f64 = 0.18;

const POPUP_GAP: f32 = 4.0;
const POPUP_PAD: f32 = 6.0;
const POPUP_CUT: f32 = 6.0;

const RGB_R: Color32 = Color32::from_rgb(255, 0, 0);
const RGB_G: Color32 = Color32::from_rgb(0, 255, 0);
const RGB_B: Color32 = Color32::from_rgb(0, 0, 255);
const GHOST_CYAN: Color32 = Color32::from_rgb(40, 220, 255);

// ─── Public API ──────────────────────────────────────────────────────
pub struct DefaultInputList;

impl InputList for DefaultInputList {
    fn render(
        ui: &mut Ui,
        t: &dyn Theme,
        value: &mut String,
        options: &[&str],
        label: &str,
    ) -> Response {
        paint(
            ui,
            value,
            options,
            label,
            t.surface(),
            (t.border(), t.brand()),
            t.text(),
            t.text_muted(),
        )
    }
}

/// Cyberpunk input list. Behaves like a text input with a floating
/// label, but reveals a filtered list below it while focused. Click a
/// row, or arrow up/down + Enter, to select. Fills `ui.available_size()`.
pub fn paint(
    ui: &mut Ui,
    value: &mut String,
    options: &[&str],
    label: &str,
    face: Color32,
    line: (Color32, Color32),
    text: Color32,
    muted: Color32,
) -> Response {
    let (outer, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer.width() <= 0.0 || outer.height() <= 0.0 {
        return response;
    }

    let id = response.id;
    let frame = Frame::from_outer(outer);

    let row_h = frame.rect.height().clamp(24.0, 40.0);

    // Pre-events filter, used for popup rect + click hit test.
    let filtered_pre = filtered_indices(options, value);
    let popup_size_pre = vec2(
        frame.rect.width().max(140.0),
        (filtered_pre.len() as f32 * row_h + POPUP_PAD * 2.0).max(row_h + POPUP_PAD * 2.0),
    );
    let screen = ui.ctx().content_rect();
    let popup_rect_pre = choose_popup_rect(frame.rect, screen, popup_size_pre);

    // ── Focus management ────────────────────────────────────────
    let was_focused = ui.ctx().memory(|m| m.has_focus(id));
    let any_click = ui.input(|i| i.pointer.any_click());
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    let clicked_inside = response.clicked();
    let clicked_outside = any_click
        && pointer_pos.map_or(true, |p| !outer.contains(p) && !popup_rect_pre.contains(p));

    if clicked_inside {
        ui.ctx().memory_mut(|m| m.request_focus(id));
    } else if clicked_outside && was_focused {
        ui.ctx().memory_mut(|m| m.surrender_focus(id));
    }

    let focused = ui.ctx().memory(|m| m.has_focus(id));

    if focused != was_focused {
        trigger_glitch(ui, &response);
        if focused {
            let t = ui.input(|i| i.time);
            ui.ctx().data_mut(|d| d.insert_temp(id.with("open_at"), t));
        }
    }

    // ── Highlighted row (keyboard nav) ─────────────────────────
    let hl_key = id.with("hl");
    let mut highlighted: Option<usize> = ui
        .ctx()
        .data(|d| d.get_temp::<Option<usize>>(hl_key))
        .flatten();

    // ── Text + keyboard events ─────────────────────────────────
    if focused {
        let mut new_text = String::new();
        let mut backspace_count = 0;
        let mut blur = false;
        let mut enter_pressed = false;
        let mut escape_pressed = false;
        let mut arrow_down = false;
        let mut arrow_up = false;

        ui.input_mut(|i| {
            i.events.retain(|event| match event {
                Event::Text(t) => {
                    new_text.push_str(t);
                    false
                }
                Event::Paste(t) => {
                    new_text.push_str(t);
                    false
                }
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                } => {
                    backspace_count += 1;
                    false
                }
                Event::Key {
                    key: Key::Enter,
                    pressed: true,
                    ..
                } => {
                    enter_pressed = true;
                    false
                }
                Event::Key {
                    key: Key::Escape,
                    pressed: true,
                    ..
                } => {
                    escape_pressed = true;
                    false
                }
                Event::Key {
                    key: Key::ArrowDown,
                    pressed: true,
                    ..
                } => {
                    arrow_down = true;
                    false
                }
                Event::Key {
                    key: Key::ArrowUp,
                    pressed: true,
                    ..
                } => {
                    arrow_up = true;
                    false
                }
                _ => true,
            });
        });

        let mut changed = false;
        if !new_text.is_empty() {
            value.push_str(&new_text);
            changed = true;
            highlighted = None;
        }
        for _ in 0..backspace_count {
            if value.pop().is_some() {
                changed = true;
                highlighted = None;
            }
        }

        // Recompute filtered after edits so arrow navigation is accurate.
        let filtered_now = filtered_indices(options, value);

        if arrow_down && !filtered_now.is_empty() {
            highlighted = Some(match highlighted {
                None => 0,
                Some(i) => (i + 1).min(filtered_now.len() - 1),
            });
        }
        if arrow_up && !filtered_now.is_empty() {
            highlighted = Some(match highlighted {
                Some(i) if i > 0 => i - 1,
                _ => 0,
            });
        }

        if enter_pressed {
            let selected = highlighted
                .filter(|i| *i < filtered_now.len())
                .map(|i| filtered_now[i]);
            if let Some(opt) = selected {
                *value = options[opt].to_string();
                changed = true;
                blur = true;
                trigger_glitch(ui, &response);
            } else {
                blur = true;
            }
            highlighted = None;
        }
        if escape_pressed {
            blur = true;
            highlighted = None;
        }

        if changed {
            response.mark_changed();
        }
        if blur {
            ui.ctx().memory_mut(|m| m.surrender_focus(id));
            trigger_glitch(ui, &response);
        }
    }

    ui.ctx().data_mut(|d| d.insert_temp(hl_key, highlighted));

    // ── Animations ──────────────────────────────────────────────
    let focus_t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("focus"), focused, FOCUS_SECS);
    let float_t = ui.ctx().animate_bool_with_time(
        response.id.with("float"),
        focused || !value.is_empty(),
        FLOAT_SECS,
    );
    let line_current = lerp(line.0, line.1, focus_t);
    let pulse = if focused {
        (ui.input(|i| i.time) * 2.2).sin() as f32 * 0.5 + 0.5
    } else {
        0.0
    };

    let motion = Motion::from_ui(ui, &response);
    let painter = ui.painter().with_clip_rect(frame.rect);

    // ── Face ────────────────────────────────────────────────────
    draw_face(&painter, &frame, face);

    if motion.hover > 0.01 {
        draw_frame_tint(&painter, &frame, line_current, 15.0 * motion.hover);
    }
    if motion.press > 0.01 {
        draw_frame_tint(&painter, &frame, line_current, 25.0 * motion.press);
    }

    // ── Border ──────────────────────────────────────────────────
    let border_width = 1.0 + focus_t * 0.8 + motion.hover * 0.5 + motion.glitch * 1.5;
    let border_alpha = 130.0 + 95.0 * focus_t + 30.0 * pulse + 40.0 * motion.hover;
    draw_border(&painter, &frame, line_current, border_width, border_alpha);

    if focus_t > 0.01 {
        draw_corner_accent(&painter, &frame, line_current, focus_t, pulse);
    }

    // ── Content ─────────────────────────────────────────────────
    let cursor_t = ui.input(|i| i.time);
    let blink = ((cursor_t * 2.0 * std::f64::consts::PI).sin() as f32 * 0.5 + 0.5).clamp(0.0, 1.0);
    let show_cursor = focused && blink > 0.10;

    draw_content(
        &painter,
        &frame,
        &motion,
        text,
        muted,
        label,
        value,
        show_cursor,
        blink,
        float_t,
    );

    if motion.glitch > 0.01 {
        draw_glitch(&painter, &frame, &motion);
        draw_scanlines(&painter, &frame, &motion);
    }

    // ── Popup ───────────────────────────────────────────────────
    // Recompute with the post-events value so the popup reflects what
    // the user sees right now.
    let focused_now = ui.ctx().memory(|m| m.has_focus(id));
    let filtered = filtered_indices(options, value);
    let open_at = ui
        .ctx()
        .data(|d| d.get_temp::<f64>(id.with("open_at")).unwrap_or(-1000.0));
    let intro = (((cursor_t - open_at) / INTRO_DURATION).clamp(0.0, 1.0)) as f32;

    if focused_now && !filtered.is_empty() {
        let popup_size = vec2(
            frame.rect.width().max(140.0),
            filtered.len() as f32 * row_h + POPUP_PAD * 2.0,
        );
        let popup_rect = choose_popup_rect(frame.rect, screen, popup_size);
        let area_id = id.with("list_area");
        let current_value = value.clone();
        let hl_snapshot = highlighted;

        let chosen: Option<usize> = Area::new(area_id)
            .order(Order::Foreground)
            .fixed_pos(popup_rect.min)
            .show(ui.ctx(), |popup_ui| -> Option<usize> {
                let pp = popup_ui.painter();

                pp.add(Shape::convex_polygon(
                    cut_polygon(popup_rect, POPUP_CUT),
                    face,
                    Stroke::NONE,
                ));
                pp.add(Shape::closed_line(
                    cut_polygon(popup_rect, POPUP_CUT),
                    Stroke::new(1.2, with_alpha(line_current, 210.0)),
                ));

                // Intro scanline sweep
                if intro < 1.0 {
                    let y = popup_rect.top() + popup_rect.height() * intro;
                    let alpha = (255.0 * (1.0 - intro)).min(255.0);
                    pp.line_segment(
                        [
                            pos2(popup_rect.left() + 3.0, y),
                            pos2(popup_rect.right() - 3.0, y),
                        ],
                        Stroke::new(1.5, with_alpha(line_current, alpha)),
                    );
                    let trail_y = y - 4.0;
                    if trail_y > popup_rect.top() {
                        pp.line_segment(
                            [
                                pos2(popup_rect.left() + 3.0, trail_y),
                                pos2(popup_rect.right() - 3.0, trail_y),
                            ],
                            Stroke::new(1.0, with_alpha(line_current, alpha * 0.4)),
                        );
                    }
                }

                let mut clicked: Option<usize> = None;
                for (row_i, &opt_index) in filtered.iter().enumerate() {
                    let opt = options[opt_index];
                    let row_rect = Rect::from_min_size(
                        pos2(
                            popup_rect.left() + POPUP_PAD,
                            popup_rect.top() + POPUP_PAD + row_i as f32 * row_h,
                        ),
                        vec2(popup_rect.width() - POPUP_PAD * 2.0, row_h),
                    );
                    let row_resp =
                        popup_ui.interact(row_rect, area_id.with(opt_index), Sense::click());
                    let is_hovered = row_resp.hovered();
                    let is_highlighted = hl_snapshot == Some(row_i);
                    let matches_value = opt == current_value;

                    if is_highlighted || is_hovered {
                        pp.add(Shape::convex_polygon(
                            cut_polygon(row_rect.shrink(1.0), 4.0),
                            with_alpha(line_current, 80.0),
                            Stroke::NONE,
                        ));
                    } else if matches_value {
                        pp.add(Shape::convex_polygon(
                            cut_polygon(row_rect.shrink(1.0), 4.0),
                            with_alpha(line_current, 40.0),
                            Stroke::NONE,
                        ));
                    }

                    let row_color = if is_highlighted || is_hovered || matches_value {
                        text
                    } else {
                        muted
                    };

                    pp.text(
                        pos2(row_rect.left() + 10.0, row_rect.center().y),
                        Align2::LEFT_CENTER,
                        opt,
                        FontId::monospace(14.0),
                        row_color,
                    );

                    if row_resp.clicked() {
                        clicked = Some(opt_index);
                    }
                }
                clicked
            })
            .inner;

        if let Some(opt_index) = chosen {
            *value = options[opt_index].to_string();
            response.mark_changed();
            ui.ctx().memory_mut(|m| m.surrender_focus(id));
            trigger_glitch(ui, &response);
            ui.ctx()
                .data_mut(|d| d.insert_temp(hl_key, Option::<usize>::None));
        }
    }

    // ── Cursor / repaint ────────────────────────────────────────
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::Text);
    }
    if focused_now {
        ui.ctx().request_repaint();
    }
    if motion.is_animating() || (0.001..0.999).contains(&focus_t) {
        ui.ctx().request_repaint();
    }

    response
}

// ─── Filtering ──────────────────────────────────────────────────────

fn filtered_indices(options: &[&str], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..options.len()).collect();
    }
    let q = query.to_lowercase();
    options
        .iter()
        .enumerate()
        .filter(|(_, o)| o.to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

// ─── Popup positioning ─────────────────────────────────────────────

fn choose_popup_rect(trigger: Rect, screen: Rect, size: Vec2) -> Rect {
    let fits = |r: Rect| -> bool {
        r.min.x >= screen.min.x
            && r.min.y >= screen.min.y
            && r.max.x <= screen.max.x
            && r.max.y <= screen.max.y
    };

    let below = Rect::from_min_size(pos2(trigger.left(), trigger.bottom() + POPUP_GAP), size);
    if fits(below) {
        return below;
    }
    let above = Rect::from_min_size(
        pos2(trigger.left(), trigger.top() - POPUP_GAP - size.y),
        size,
    );
    if fits(above) {
        return above;
    }
    let right = Rect::from_min_size(pos2(trigger.right() + POPUP_GAP, trigger.top()), size);
    if fits(right) {
        return right;
    }
    let left = Rect::from_min_size(
        pos2(trigger.left() - POPUP_GAP - size.x, trigger.top()),
        size,
    );
    if fits(left) {
        return left;
    }

    let x = trigger
        .left()
        .clamp(screen.min.x, (screen.max.x - size.x).max(screen.min.x));
    let y = trigger
        .bottom()
        .clamp(screen.min.y, (screen.max.y - size.y).max(screen.min.y));
    Rect::from_min_size(pos2(x, y), size)
}

// ─── Frame / Motion ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Frame {
    rect: Rect,
    corner: f32,
    short: f32,
}

impl Frame {
    fn from_outer(outer: Rect) -> Self {
        let pad = FACE_PAD
            .min(outer.width() * 0.10)
            .min(outer.height() * 0.20);
        let rect = outer.shrink(pad);
        let short = rect.width().min(rect.height());
        let corner = CORNER_CUT.min(short * 0.28).max(0.0);
        Self {
            rect,
            corner,
            short,
        }
    }
}

#[derive(Clone, Copy)]
struct Motion {
    time: f64,
    hover: f32,
    press: f32,
    glitch: f32,
}

impl Motion {
    fn from_ui(ui: &Ui, response: &Response) -> Self {
        let time = ui.input(|i| i.time);
        let hover =
            ui.ctx()
                .animate_bool_with_time(response.id.with("h"), response.hovered(), HOVER_SECS);
        let press = ui.ctx().animate_bool_with_time(
            response.id.with("p"),
            response.is_pointer_button_down_on(),
            PRESS_SECS,
        );
        let glitch = glitch_intensity(ui, response, time);
        Self {
            time,
            hover,
            press,
            glitch,
        }
    }

    fn is_animating(&self) -> bool {
        self.glitch > 0.001
            || (0.001..0.999).contains(&self.hover)
            || (0.001..0.999).contains(&self.press)
    }
}

// ─── Content — floating label + value + cursor ─────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_content(
    painter: &Painter,
    frame: &Frame,
    motion: &Motion,
    text: Color32,
    muted: Color32,
    label: &str,
    value: &str,
    show_cursor: bool,
    blink: f32,
    float_t: f32,
) {
    let body_size = (frame.rect.height() * 0.42).clamp(10.0, 18.0);
    let small_size = (body_size * 0.72).clamp(8.0, 12.0);
    let label_size = body_size + (small_size - body_size) * float_t;
    let label_font = FontId::monospace(label_size);
    let value_font = FontId::monospace(body_size);

    let cy = frame.rect.center().y;
    let top = frame.rect.top();
    let h = frame.rect.height();
    let pad = 12.0;
    let x = frame.rect.left() + pad;

    let floated_y = top + label_size * 0.85;
    let label_y = cy + (floated_y - cy) * float_t;
    let value_y = top + h * 0.68;

    let label_color = lerp(muted, text, 0.35 * float_t);

    let lg = painter.layout_no_wrap(label.to_string(), label_font, label_color);
    let label_w = lg.size().x;
    let label_h = lg.size().y;
    painter.galley(pos2(x, label_y - label_h * 0.5), lg, label_color);

    if float_t > 0.01 {
        let line_y = label_y + label_size * 0.55;
        painter.line_segment(
            [pos2(x, line_y), pos2(x + label_w, line_y)],
            Stroke::new(1.0, with_alpha(label_color, 90.0 * float_t)),
        );
    }

    let value_color = with_alpha(text, 255.0 * float_t);
    let value_width = if value.is_empty() {
        0.0
    } else {
        let vg = painter.layout_no_wrap(value.to_string(), value_font.clone(), value_color);
        let w = vg.size().x;
        let vh = vg.size().y;

        if motion.glitch > 0.01 {
            let aberr = 1.0 + motion.glitch * 3.0;
            let alpha = 180.0 * motion.glitch * float_t;
            painter.text(
                pos2(x - aberr, value_y),
                Align2::LEFT_CENTER,
                value,
                value_font.clone(),
                with_alpha(RGB_R, alpha),
            );
            painter.text(
                pos2(x + aberr, value_y),
                Align2::LEFT_CENTER,
                value,
                value_font.clone(),
                with_alpha(GHOST_CYAN, alpha),
            );
        }

        painter.galley(pos2(x, value_y - vh * 0.5), vg, value_color);
        w
    };

    if show_cursor {
        let cursor_x = x + value_width + 1.0;
        let cursor_color = with_alpha(text, 255.0 * blink * float_t);
        painter.line_segment(
            [
                pos2(cursor_x, value_y - body_size * 0.55),
                pos2(cursor_x, value_y + body_size * 0.55),
            ],
            Stroke::new(1.5, cursor_color),
        );
    }
}

// ─── Corner accent ─────────────────────────────────────────────────

fn draw_corner_accent(painter: &Painter, frame: &Frame, color: Color32, t: f32, pulse: f32) {
    let cut = frame.corner;
    if cut < 3.0 {
        return;
    }
    let offset = (frame.short * 0.10).clamp(1.5, 5.0);
    let inset = (cut * 0.35).max(1.0);
    let p2 = pos2(frame.rect.right() - cut, frame.rect.top());
    let p3 = pos2(frame.rect.right(), frame.rect.top() + cut);
    let k = offset * 0.7071;
    let p2i = pos2(p2.x - k, p2.y + k);
    let p3i = pos2(p3.x - k, p3.y + k);
    let dx = p3i.x - p2i.x;
    let dy = p3i.y - p2i.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < inset * 2.0 + 1.0 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let a = pos2(p2i.x + ux * inset, p2i.y + uy * inset);
    let b = pos2(p3i.x - ux * inset, p3i.y - uy * inset);
    let width = (1.5 + pulse * 1.0) * t;
    let alpha = (200.0 + 40.0 * pulse) * t;
    painter.line_segment([a, b], Stroke::new(width, with_alpha(color, alpha)));
}

// ─── Glitch ─────────────────────────────────────────────────────────

fn draw_glitch(painter: &Painter, frame: &Frame, motion: &Motion) {
    let w = frame.rect.width();
    let h = frame.rect.height();
    let h_count = ((w * 0.08) as usize).clamp(2, 12);
    let p_count = ((w * h) / 500.0) as usize;

    for i in 0..h_count {
        if !flicker(motion, 1.0, 10 + i as u64) {
            continue;
        }
        let y = frame.rect.top() + rnd01(motion, 25.0, i) * h;
        let x1 = frame.rect.left() + rnd01(motion, 30.0, i + 100) * w;
        let x2 = frame.rect.left() + rnd01(motion, 30.0, i + 200) * w;
        let (sx, ex) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        if ex - sx < 2.0 {
            continue;
        }
        painter.rect_filled(
            Rect::from_min_size(pos2(sx, y), vec2(ex - sx, 1.0)),
            0.0,
            with_alpha(pick_rgb(i), motion.glitch * 180.0),
        );
    }

    for i in 0..p_count {
        if !flicker(motion, 0.8, 5000 + i as u64) {
            continue;
        }
        let x = frame.rect.left() + rnd01(motion, 40.0, i + 300) * w;
        let y = frame.rect.top() + rnd01(motion, 40.0, i + 400) * h;
        painter.rect_filled(
            Rect::from_min_size(pos2(x, y), vec2(1.0, 1.0)),
            0.0,
            with_alpha(pick_rgb(i), motion.glitch * 180.0),
        );
    }
}

fn draw_scanlines(painter: &Painter, frame: &Frame, motion: &Motion) {
    let base = (frame.rect.height() / 14.0).clamp(2.0, 6.0);
    let count = ((base * motion.glitch).ceil() as usize).max(1);
    let speed = 70.0 + motion.glitch as f64 * 200.0;
    let step = frame.rect.height() / count as f32;
    let alpha = (180.0 * motion.glitch).min(255.0);
    let width = 1.0 + motion.glitch * 1.0;

    for i in 0..count {
        let base_y = frame.rect.top()
            + ((motion.time * speed + i as f64 * step as f64) as f32)
                .rem_euclid(frame.rect.height());
        let jy = rnd(motion, 60.0, i) * 18.0 * motion.glitch;
        let jx = rnd(motion, 40.0, i + 10) * 20.0 * motion.glitch;
        painter.line_segment(
            [
                pos2(frame.rect.left() + 4.0 + jx, base_y + jy),
                pos2(frame.rect.right() - 4.0 + jx, base_y + jy),
            ],
            Stroke::new(width, with_alpha(Color32::WHITE, alpha)),
        );
    }
}

fn pick_rgb(i: usize) -> Color32 {
    match i % 3 {
        0 => RGB_R,
        1 => RGB_G,
        _ => RGB_B,
    }
}

// ─── Glitch trigger / intensity ───────────────────────────────────

fn trigger_glitch(ui: &mut Ui, response: &Response) {
    let t = ui.input(|i| i.time);
    let key = response.id.with("glitch_at");
    ui.ctx().data_mut(|d| d.insert_temp(key, t));
}

fn glitch_intensity(ui: &Ui, response: &Response, time: f64) -> f32 {
    let key = response.id.with("glitch_at");
    let last = ui.ctx().data(|d| d.get_temp::<f64>(key)).unwrap_or(-1000.0);
    let t = ((time - last) / GLITCH_DURATION).clamp(0.0, 1.0) as f32;
    if t < 1.0 {
        (1.0 - t).powi(2)
    } else {
        0.0
    }
}

// ─── Paint primitives ──────────────────────────────────────────────

fn draw_face(painter: &Painter, frame: &Frame, fill: Color32) {
    painter.add(Shape::convex_polygon(
        cut_polygon(frame.rect, frame.corner),
        fill,
        Stroke::NONE,
    ));
}

fn draw_frame_tint(painter: &Painter, frame: &Frame, color: Color32, alpha: f32) {
    painter.add(Shape::convex_polygon(
        cut_polygon(frame.rect, frame.corner),
        with_alpha(color, alpha),
        Stroke::NONE,
    ));
}

fn draw_border(painter: &Painter, frame: &Frame, color: Color32, width: f32, alpha: f32) {
    painter.add(Shape::closed_line(
        cut_polygon(frame.rect, frame.corner),
        Stroke::new(width, with_alpha(color, alpha)),
    ));
}

// ─── Helpers ────────────────────────────────────────────────────────

fn flicker(motion: &Motion, mult: f32, seed: u64) -> bool {
    if motion.glitch * mult <= 0.01 {
        return false;
    }
    let n = noise((motion.time * 30.0) as u64, seed as usize) + 0.5;
    n < motion.glitch * mult * 1.2
}

fn rnd(motion: &Motion, speed: f64, seed: usize) -> f32 {
    noise((motion.time * speed) as u64, seed)
}

fn rnd01(motion: &Motion, speed: f64, seed: usize) -> f32 {
    rnd(motion, speed, seed) + 0.5
}

fn cut_polygon(rect: Rect, cut: f32) -> Vec<Pos2> {
    let cut = cut
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);
    vec![
        pos2(rect.left() + cut, rect.top()),
        pos2(rect.right() - cut, rect.top()),
        pos2(rect.right(), rect.top() + cut),
        pos2(rect.right(), rect.bottom() - cut),
        pos2(rect.right() - cut, rect.bottom()),
        pos2(rect.left() + cut, rect.bottom()),
        pos2(rect.left(), rect.bottom() - cut),
        pos2(rect.left(), rect.top() + cut),
    ]
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        alpha.clamp(0.0, 255.0) as u8,
    )
}

fn noise(seed: u64, i: usize) -> f32 {
    let mut h = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    h ^= i as u64;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((h >> 33) as f32 / (u32::MAX >> 1) as f32) - 0.5
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}
