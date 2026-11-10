use prelude::DropdownButton;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Area;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
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
use prelude::egui::pos2;
use prelude::egui::vec2;

// ─── Tuning ──────────────────────────────────────────────────────────
const FACE_PAD: f32 = 6.0;
const CORNER_CUT: f32 = 8.0;
const HOVER_SECS: f32 = 0.16;
const PRESS_SECS: f32 = 0.05;
const POPUP_GAP: f32 = 4.0;
const POPUP_PAD: f32 = 6.0;
const POPUP_CUT: f32 = 6.0;

const GLITCH_DURATION: f64 = 0.22;
const INTRO_DURATION: f64 = 0.18;

const RGB_R: Color32 = Color32::from_rgb(255, 0, 0);
const RGB_G: Color32 = Color32::from_rgb(0, 255, 0);
const RGB_B: Color32 = Color32::from_rgb(0, 0, 255);
const GHOST_CYAN: Color32 = Color32::from_rgb(40, 220, 255);

// ─── Public API ──────────────────────────────────────────────────────
pub struct DefaultDropdownButton;

impl DropdownButton for DefaultDropdownButton {
    fn render(
        ui: &mut Ui,
        t: &dyn Theme,
        selected: &mut usize,
        options: &[&str],
        label: &str,
    ) -> Response {
        paint(
            ui,
            selected,
            options,
            label,
            t.surface(),
            (t.border(), t.brand()),
            t.text(),
            t.text_muted(),
        )
    }
}

/// Cyberpunk dropdown with glitch bursts, chromatic-aberration label,
/// and a popup-intro scanline sweep.
pub fn paint(
    ui: &mut Ui,
    selected: &mut usize,
    options: &[&str],
    label: &str,
    face: Color32,
    line: (Color32, Color32),
    text: Color32,
    muted: Color32,
) -> Response {
    let (outer, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer.width() <= 0.0 || outer.height() <= 0.0 || options.is_empty() {
        return response;
    }
    if *selected >= options.len() {
        *selected = 0;
    }

    let open_key = response.id.with("dd_open");
    let open_at_key = response.id.with("dd_open_at");
    let mut open: bool = ui.ctx().data(|d| d.get_temp(open_key).unwrap_or(false));

    let frame = Frame::from_outer(outer);
    let motion = Motion::from_ui(ui, &response);
    let line_current = if open { line.1 } else { line.0 };
    let pulse = if open {
        (motion.time * 2.2).sin() as f32 * 0.5 + 0.5
    } else {
        0.0
    };

    // ── Trigger ────────────────────────────────────────────────
    {
        let painter = ui.painter().with_clip_rect(frame.rect);
        draw_face(&painter, &frame, face);

        let tint_alpha = 15.0 * motion.hover + if open { 12.0 } else { 0.0 };
        if tint_alpha > 0.01 {
            draw_frame_tint(&painter, &frame, line_current, tint_alpha);
        }

        let border_alpha =
            130.0 + 40.0 * motion.hover + if open { 90.0 + 30.0 * pulse } else { 0.0 };
        let border_width = 1.0 + motion.hover * 0.5 + if open { 0.8 + 0.3 * pulse } else { 0.0 };
        draw_border(&painter, &frame, line_current, border_width, border_alpha);

        if open {
            draw_corner_accent(&painter, &frame, line_current, pulse);
        }

        draw_trigger(
            &painter,
            &frame,
            text,
            muted,
            label,
            options[*selected],
            open,
            motion.glitch,
        );

        // Glitch burst overlay
        if motion.glitch > 0.01 {
            draw_glitch(&painter, &frame, &motion);
            draw_scanlines(&painter, &frame, &motion);
        }
    }

    // ── Toggle open ────────────────────────────────────────────
    if response.clicked() {
        open = !open;
        trigger_glitch(ui, &response);
        let t = ui.input(|i| i.time);
        ui.ctx().data_mut(|d| d.insert_temp(open_at_key, t));
    }

    // ── Popup ──────────────────────────────────────────────────
    if open {
        let screen = ui.ctx().content_rect();
        let row_h = frame.rect.height().clamp(24.0, 40.0);
        let popup_size = vec2(
            frame.rect.width().max(140.0),
            options.len() as f32 * row_h + POPUP_PAD * 2.0,
        );
        let popup_rect = choose_popup_rect(frame.rect, screen, popup_size);

        // Popup intro animation progress (0 = just opened, 1 = settled)
        let open_at = ui
            .ctx()
            .data(|d| d.get_temp::<f64>(open_at_key).unwrap_or(-1000.0));
        let intro = (((motion.time - open_at) / INTRO_DURATION).clamp(0.0, 1.0)) as f32;

        let area_id = response.id.with("dd_area");
        let current_sel = *selected;
        let row_h_local = row_h;

        let inner = Area::new(area_id)
            .order(Order::Foreground)
            .fixed_pos(popup_rect.min)
            .show(ui.ctx(), |popup_ui| -> Option<usize> {
                let pp = popup_ui.painter();

                // Popup face + border
                pp.add(Shape::convex_polygon(
                    cut_polygon(popup_rect, POPUP_CUT),
                    face,
                    Stroke::NONE,
                ));
                pp.add(Shape::closed_line(
                    cut_polygon(popup_rect, POPUP_CUT),
                    Stroke::new(1.2, with_alpha(line_current, 210.0)),
                ));

                // Intro scanline sweep (top → bottom)
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
                    // Trailing ghost line
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

                let mut chosen: Option<usize> = None;
                for (i, opt) in options.iter().enumerate() {
                    let row_rect = Rect::from_min_size(
                        pos2(
                            popup_rect.left() + POPUP_PAD,
                            popup_rect.top() + POPUP_PAD + i as f32 * row_h_local,
                        ),
                        vec2(popup_rect.width() - POPUP_PAD * 2.0, row_h_local),
                    );
                    let row_resp = popup_ui.interact(row_rect, area_id.with(i), Sense::click());
                    let is_selected = i == current_sel;
                    let is_hovered = row_resp.hovered();

                    if is_hovered && !is_selected {
                        pp.add(Shape::convex_polygon(
                            cut_polygon(row_rect.shrink(1.0), 4.0),
                            with_alpha(line_current, 50.0),
                            Stroke::NONE,
                        ));
                    }
                    if is_selected {
                        pp.add(Shape::convex_polygon(
                            cut_polygon(row_rect.shrink(1.0), 4.0),
                            with_alpha(line_current, 110.0),
                            Stroke::NONE,
                        ));
                    }

                    let row_color = if is_selected {
                        text
                    } else if is_hovered {
                        lerp(muted, text, 0.6)
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
                        chosen = Some(i);
                    }
                }
                chosen
            });
        let chosen = inner.inner;

        // Outside-click detection
        let pointer = ui.ctx().input(|i| i.pointer.interact_pos());
        let any_click = ui.ctx().input(|i| i.pointer.any_click());
        let clicked_outside = any_click
            && pointer.map_or(false, |p| {
                !popup_rect.contains(p) && !frame.rect.contains(p)
            });

        if let Some(i) = chosen {
            if i != *selected {
                *selected = i;
                response.mark_changed();
            }
            open = false;
            trigger_glitch(ui, &response);
        } else if clicked_outside {
            open = false;
            trigger_glitch(ui, &response);
        }

        ui.ctx().request_repaint();
    }

    ui.ctx().data_mut(|d| d.insert_temp(open_key, open));

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        ui.ctx().request_repaint();
    }
    // Compute intro-active outside any lock. `motion.time` was captured
    // once in `Motion::from_ui` — do not re-enter `ui.input()` here.
    let intro_active = if open {
        let at = ui
            .ctx()
            .data(|d| d.get_temp::<f64>(open_at_key).unwrap_or(-1000.0));
        (motion.time - at) < INTRO_DURATION
    } else {
        false
    };

    if motion.is_animating() || (0.001..0.999).contains(&pulse) || intro_active {
        ui.ctx().request_repaint();
    }

    response
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

// ─── Trigger label with chromatic aberration ───────────────────────

#[allow(clippy::too_many_arguments)]
fn draw_trigger(
    painter: &Painter,
    frame: &Frame,
    text: Color32,
    muted: Color32,
    label: &str,
    value: &str,
    open: bool,
    glitch: f32,
) {
    let pad = 10.0;
    let chevron_size = 8.0;
    let font_size = (frame.rect.height() * 0.45).clamp(10.0, 18.0);
    let font = FontId::monospace(font_size);
    let cy = frame.rect.center().y;
    let mut x = frame.rect.left() + pad;

    if !label.is_empty() {
        let galley = painter.layout_no_wrap(label.to_string(), font.clone(), muted);
        let h = galley.size().y;
        painter.galley(pos2(x, cy - h * 0.5), galley.clone(), muted);
        x += galley.size().x + 6.0;
    }

    let value_color = if open { text } else { lerp(muted, text, 0.5) };

    // Chromatic ghosts on glitch
    if glitch > 0.01 {
        let aberr = 1.0 + glitch * 3.0;
        let alpha = 200.0 * glitch;
        painter.text(
            pos2(x - aberr, cy),
            Align2::LEFT_CENTER,
            value,
            font.clone(),
            with_alpha(RGB_R, alpha),
        );
        painter.text(
            pos2(x + aberr, cy),
            Align2::LEFT_CENTER,
            value,
            font.clone(),
            with_alpha(GHOST_CYAN, alpha),
        );
    }

    painter.text(pos2(x, cy), Align2::LEFT_CENTER, value, font, value_color);

    // Chevron on the right
    let chevron_pos = pos2(frame.rect.right() - pad - chevron_size * 0.5, cy);
    draw_chevron(
        painter,
        chevron_pos,
        chevron_size,
        if open { text } else { muted },
        open,
    );
}

fn draw_chevron(painter: &Painter, center: Pos2, size: f32, color: Color32, up: bool) {
    let half = size * 0.5;
    let h = size * 0.3;
    let (a, b, c) = if up {
        (
            pos2(center.x - half, center.y + h * 0.5),
            pos2(center.x, center.y - h * 0.5),
            pos2(center.x + half, center.y + h * 0.5),
        )
    } else {
        (
            pos2(center.x - half, center.y - h * 0.5),
            pos2(center.x, center.y + h * 0.5),
            pos2(center.x + half, center.y - h * 0.5),
        )
    };
    let stroke = Stroke::new(1.5, color);
    painter.line_segment([a, b], stroke);
    painter.line_segment([b, c], stroke);
}

// ─── Corner accent ─────────────────────────────────────────────────

fn draw_corner_accent(painter: &Painter, frame: &Frame, color: Color32, pulse: f32) {
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
    let width = 1.5 + pulse * 1.0;
    let alpha = 200.0 + 40.0 * pulse;
    painter.line_segment([a, b], Stroke::new(width, with_alpha(color, alpha)));
}

// ─── Glitch (same technique as the button) ────────────────────────

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
    if t < 1.0 { (1.0 - t).powi(2) } else { 0.0 }
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
