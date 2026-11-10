use prelude::Theme;
use prelude::ToggleButton;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Shape;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

// ─── Tuning ──────────────────────────────────────────────────────────
const FACE_PAD: f32 = 6.0;
const CORNER_CUT: f32 = 8.0;
const GLITCH_DURATION: f64 = 0.25;
const HOVER_SECS: f32 = 0.16;
const PRESS_SECS: f32 = 0.05;
const TOGGLE_SECS: f32 = 0.22;

const RGB_R: Color32 = Color32::from_rgb(255, 0, 0);
const RGB_G: Color32 = Color32::from_rgb(0, 255, 0);
const RGB_B: Color32 = Color32::from_rgb(0, 0, 255);
const GHOST_CYAN: Color32 = Color32::from_rgb(40, 220, 255);

// ─── Public API ──────────────────────────────────────────────────────
pub struct DefaultToggleButton;

impl ToggleButton for DefaultToggleButton {
    fn render(ui: &mut Ui, t: &dyn Theme, value: &mut bool, label: &str) -> Response {
        paint(
            ui,
            value,
            t.surface(),
            (t.border(), t.brand()),
            t.text(),
            t.text_muted(),
            label,
        )
    }
}

/// Cyberpunk toggle — single label, no indicator. The state is
/// communicated by face tint, pulsing border, label color, and a
/// corner accent. Fills `ui.available_size()`.
///
/// Colors injected by the caller:
///   * `face`  — background
///   * `line`  — (off, on) accent for border, face tint, corner accent
///   * `text`  — bright label color (on)
///   * `muted` — dim label color (off)
pub fn paint(
    ui: &mut Ui,
    value: &mut bool,
    face: Color32,
    line: (Color32, Color32),
    text: Color32,
    muted: Color32,
    label: &str,
) -> Response {
    let (outer, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer.width() <= 0.0 || outer.height() <= 0.0 {
        return response;
    }

    if response.clicked() {
        *value = !*value;
        trigger_glitch(ui, &response);
        response.mark_changed();
    }

    let frame = Frame::from_outer(outer);
    let motion = Motion::from_ui(ui, &response);
    let painter = ui.painter().with_clip_rect(frame.rect);

    // t: 0 = off, 1 = on. Drives every visual difference between states.
    let t = ui
        .ctx()
        .animate_bool_with_time(response.id.with("t"), *value, TOGGLE_SECS);

    let line_on = line.1;
    let line_off = line.0;

    // Slow pulse only while on — a "powered" feeling.
    let pulse = ((motion.time * 2.2).sin() as f32 * 0.5 + 0.5) * t;

    // ── Face: plain surface → slightly tinted ────────────────────
    let tinted_face = lerp(face, line_on, 0.10 * t);
    draw_face(&painter, &frame, tinted_face);

    // ── Hover / press tint ──────────────────────────────────────
    let hover_color = lerp(line_off, line_on, t);
    if motion.hover > 0.01 {
        draw_frame_tint(&painter, &frame, hover_color, 15.0 * motion.hover);
    }
    if motion.press > 0.01 {
        draw_frame_tint(&painter, &frame, hover_color, 25.0 * motion.press);
    }

    // ── Border: dim → bright, thin → thick, + pulse ─────────────
    let border_color = lerp(line_off, line_on, t);
    let border_width = 1.0 + t * 0.9 + motion.hover * 0.5 + motion.glitch * 1.5;
    let border_alpha = 120.0 + 95.0 * t + 30.0 * pulse + 40.0 * motion.hover;
    draw_border(&painter, &frame, border_color, border_width, border_alpha);

    // ── Corner accent: appears only when on ─────────────────────
    if t > 0.01 {
        draw_corner_accent(&painter, &frame, line_on, t, pulse);
    }

    // ── Label: muted → bright ───────────────────────────────────
    draw_label(&painter, &frame, &motion, text, muted, label, t);

    // ── Glitch on click ─────────────────────────────────────────
    if motion.glitch > 0.01 {
        draw_glitch(&painter, &frame, &motion);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if response.hovered() || motion.is_animating() || (0.001..0.999).contains(&t) {
        ui.ctx().request_repaint();
    }

    response
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
        self.glitch > 0.001 || (0.001..0.999).contains(&self.hover)
    }
}

// ─── Corner accent ─────────────────────────────────────────────────

/// Short diagonal slash parallel to the top-right cut. Sits entirely
/// inside the polygon, scales with `frame.short`, and brightens with
/// the same pulse as the border.
fn draw_corner_accent(painter: &Painter, frame: &Frame, color: Color32, t: f32, pulse: f32) {
    let cut = frame.corner;
    if cut < 3.0 {
        return;
    }

    // Perpendicular inset from the cut line — small, proportional.
    let offset = (frame.short * 0.10).clamp(1.5, 5.0);
    let inset = (cut * 0.35).max(1.0);

    // Cut line endpoints (top-right corner, from cut_polygon).
    let p2 = pos2(frame.rect.right() - cut, frame.rect.top());
    let p3 = pos2(frame.rect.right(), frame.rect.top() + cut);

    // Move both endpoints toward the button interior (down-left).
    let k = offset * 0.7071;
    let p2i = pos2(p2.x - k, p2.y + k);
    let p3i = pos2(p3.x - k, p3.y + k);

    // Shorten each end by `inset`.
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

// ─── Label ─────────────────────────────────────────────────────────

fn draw_label(
    painter: &Painter,
    frame: &Frame,
    motion: &Motion,
    active: Color32,
    muted: Color32,
    label: &str,
    t: f32,
) {
    let hover_boost = motion.hover.max(motion.press);
    let blend = (t * 0.85 + hover_boost * 0.15).min(1.0);
    let text_color = lerp(muted, active, blend);

    // Font sized to fit the button — proportional, never overflows.
    let avail_w = (frame.rect.width() * 0.82).max(4.0);
    let avail_h = (frame.rect.height() * 0.55).max(4.0);
    let h_font = avail_h.clamp(8.0, 20.0);
    let char_count = label.chars().count().max(1) as f32;
    let w_font = (avail_w / (char_count * 0.6)).clamp(8.0, 20.0);
    let font_size = h_font.min(w_font);
    let font = FontId::monospace(font_size);
    let pos = frame.rect.center();

    // On + hover → subtle brand-colored halo (4-direction offset text).
    if t > 0.01 && hover_boost > 0.01 {
        let halo = with_alpha(active, 80.0 * t * hover_boost);
        for off in [
            vec2(1.0, 0.0),
            vec2(-1.0, 0.0),
            vec2(0.0, 1.0),
            vec2(0.0, -1.0),
        ] {
            painter.text(pos + off, Align2::CENTER_CENTER, label, font.clone(), halo);
        }
    }

    // Glitch ghost — R/G/B split, only during the click burst.
    if motion.glitch > 0.01 {
        let aberr = 1.0 + motion.glitch * 3.0;
        let alpha = 200.0 * motion.glitch;
        painter.text(
            pos - vec2(aberr, 0.0),
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            with_alpha(RGB_R, alpha),
        );
        painter.text(
            pos + vec2(aberr, 0.0),
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            with_alpha(GHOST_CYAN, alpha),
        );
    }

    painter.text(pos, Align2::CENTER_CENTER, label, font, text_color);
}

// ─── Paint primitives (same cut-corner shape as the button) ────────

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

fn pick_rgb(i: usize) -> Color32 {
    match i % 3 {
        0 => RGB_R,
        1 => RGB_G,
        _ => RGB_B,
    }
}

// ─── Glitch trigger ────────────────────────────────────────────────

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
