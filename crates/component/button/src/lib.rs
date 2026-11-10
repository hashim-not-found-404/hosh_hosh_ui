use prelude::PrimaryButton;
use prelude::SecondaryButton;
use prelude::Theme;
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
const FACE_PAD: f32 = 10.0;
const CORNER_CUT: f32 = 10.0;
const GLITCH_DURATION: f64 = 0.20;
const HOVER_SECS: f32 = 0.20;
const PRESS_SECS: f32 = 0.05;

// Fixed RGB split colors for the glitch effect — theme-independent so
// they always pop, regardless of the face color underneath.
const RGB_R: Color32 = Color32::from_rgb(255, 0, 0);
const RGB_G: Color32 = Color32::from_rgb(0, 255, 0);
const RGB_B: Color32 = Color32::from_rgb(0, 0, 255);
const GHOST_CYAN: Color32 = Color32::from_rgb(40, 220, 255);

// ─── Public API ──────────────────────────────────────────────────────
pub struct DefaultPrimaryButton;
pub struct DefaultSecondaryButton;

impl PrimaryButton for DefaultPrimaryButton {
    fn render(ui: &mut Ui, t: &dyn Theme, label: &str) -> Response {
        // Bold: face == line (a solid accent block), text is body text.
        paint(ui, t.brand(), t.brand(), t.text(), label)
    }
}

impl SecondaryButton for DefaultSecondaryButton {
    fn render(ui: &mut Ui, t: &dyn Theme, label: &str) -> Response {
        // Subtle: face is an elevated surface, line is the alternate
        // brand accent, text is the body text color — all three distinct.
        paint(ui, t.surface_elevated(), t.brand_alt(), t.text(), label)
    }
}
/// The whole button. Three colors are injected by the caller:
///   * `face` — the background fill
///   * `line` — border, tape corners, hover tint, glow, glitch accent
///   * `text` — label text and its halo
///
/// The theme never leaks in here. The caller decides which tokens to
/// pass based on the visual language it wants.
fn paint(ui: &mut Ui, face: Color32, line: Color32, text: Color32, label: &str) -> Response {
    let (outer, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer.width() <= 0.0 || outer.height() <= 0.0 {
        return response;
    }

    let frame = Frame::from_outer(outer);
    let motion = Motion::from_ui(ui, &response);
    let painter = ui.painter().with_clip_rect(outer);

    // ── Face fill. ────────────────────────────────────────────────
    draw_face(&painter, &frame, face);

    // ── Hover / press tint (line color over the face). ────────────
    if motion.hover > 0.01 {
        draw_tint(&painter, &frame, line, 22.0 * motion.hover);
    }
    if motion.press > 0.01 {
        draw_tint(&painter, &frame, line, 32.0 * motion.press);
    }

    // ── Subtle glow, only while active. ───────────────────────────
    if motion.hover > 0.01 || motion.glitch > 0.01 {
        draw_glow(&painter, &frame, line, motion.glow_intensity() * 0.45);
    }

    // ── Tape corners. ─────────────────────────────────────────────
    draw_tape(&painter, &frame, line, 180.0 + 75.0 * motion.hover);

    // ── Border. ───────────────────────────────────────────────────
    draw_border(
        &painter,
        &frame,
        line,
        1.0 + motion.hover * 1.0 + motion.glitch * 1.5,
        160.0 + 95.0 * motion.hover,
    );

    // ── Glitch (fixed R/G/B, pops on any face). ───────────────────
    if motion.glitch > 0.01 {
        draw_glitch(&painter, &frame, &motion);
    }

    // ── Label. ────────────────────────────────────────────────────
    draw_label(&painter, &frame, &motion, text, label);

    // ── Scanlines, only during glitch. ────────────────────────────
    if motion.glitch > 0.01 {
        draw_scanlines(&painter, &frame, &motion);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if response.hovered() || motion.is_animating() {
        ui.ctx().request_repaint();
    }

    response
}

// ─── Context types ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Frame {
    rect: Rect,
    corner: f32,
    short: f32,
    scale: f32,
}

impl Frame {
    fn from_outer(outer: Rect) -> Self {
        let pad = FACE_PAD
            .min(outer.width() * 0.20)
            .min(outer.height() * 0.20);
        let rect = outer.shrink(pad);
        let short = rect.width().min(rect.height());
        let corner = CORNER_CUT.min(short * 0.28).max(0.0);
        let scale = (rect.height() / 40.0).clamp(0.3, 1.0);
        Self {
            rect,
            corner,
            short,
            scale,
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

    fn glow_intensity(&self) -> f32 {
        let pulse = ((self.time * 2.5).sin() as f32 * 0.5 + 0.5) * self.hover;
        0.35 + 0.65 * self.hover + 0.15 * pulse + 0.4 * self.glitch
    }

    fn is_animating(&self) -> bool {
        self.glitch > 0.001 || (0.001..0.999).contains(&self.hover)
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

fn draw_tint(painter: &Painter, frame: &Frame, color: Color32, alpha: f32) {
    painter.add(Shape::convex_polygon(
        cut_polygon(frame.rect, frame.corner),
        with_alpha(color, alpha),
        Stroke::NONE,
    ));
}

fn draw_glow(painter: &Painter, frame: &Frame, color: Color32, intensity: f32) {
    let layers = [
        ((frame.short * 0.30).min(12.0), 0.06_f32),
        ((frame.short * 0.18).min(7.0), 0.14),
        ((frame.short * 0.08).min(3.0), 0.22),
    ];
    for (expand, mult) in layers {
        let alpha = (mult * intensity * 255.0).min(255.0);
        painter.add(Shape::convex_polygon(
            cut_polygon(frame.rect.expand(expand), frame.corner + expand),
            with_alpha(color, alpha),
            Stroke::NONE,
        ));
    }
}

fn draw_tape(painter: &Painter, frame: &Frame, color: Color32, alpha: f32) {
    let stroke = Stroke::new(2.0, with_alpha(color, alpha));
    let tape_len = (frame.short * 0.35).min(CORNER_CUT + 10.0).max(2.0);
    let step = (tape_len * 0.2).max(1.0);
    let inset = (tape_len * 0.15).max(1.5);

    let (l, r, t, b) = (
        frame.rect.left(),
        frame.rect.right(),
        frame.rect.top(),
        frame.rect.bottom(),
    );
    for i in 0..3 {
        let off = i as f32 * step;
        if tape_len - off < 1.5 {
            break;
        }
        painter.line_segment(
            [
                pos2(l + inset + off, t + inset + tape_len - off),
                pos2(l + inset + tape_len - off, t + inset + off),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(r - inset - off, b - inset - tape_len + off),
                pos2(r - inset - tape_len + off, b - inset - off),
            ],
            stroke,
        );
    }
}

fn draw_border(painter: &Painter, frame: &Frame, color: Color32, width: f32, alpha: f32) {
    painter.add(Shape::closed_line(
        cut_polygon(frame.rect, frame.corner),
        Stroke::new(width, with_alpha(color, alpha)),
    ));
}

// ─── Sparse glitch — 1px R/G/B lines + pixels ──────────────────────

fn draw_glitch(painter: &Painter, frame: &Frame, motion: &Motion) {
    let w = frame.rect.width();
    let h = frame.rect.height();
    let area = w * h;

    let h_count = ((w * 0.10) as usize).clamp(2, 20);
    let p_count = (area / 400.0) as usize;

    glitch_horizontal_lines(painter, frame, motion, h_count);
    glitch_pixels(painter, frame, motion, p_count);
}

fn glitch_horizontal_lines(painter: &Painter, frame: &Frame, motion: &Motion, count: usize) {
    let l = frame.rect.left();
    let w = frame.rect.width();
    let t = frame.rect.top();
    let h = frame.rect.height();

    for i in 0..count {
        if !flicker(motion, 1.0, 10 + i as u64) {
            continue;
        }
        let y = t + rnd01(motion, 25.0, i) * h;
        let x1 = l + rnd01(motion, 30.0, i + 100) * w;
        let x2 = l + rnd01(motion, 30.0, i + 200) * w;
        let (sx, ex) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
        if ex - sx < 2.0 {
            continue;
        }
        let color = pick_rgb(i);
        fill(
            painter,
            sx,
            y,
            ex - sx,
            1.0,
            with_alpha(color, motion.glitch * 200.0),
        );
    }
}

fn glitch_pixels(painter: &Painter, frame: &Frame, motion: &Motion, count: usize) {
    for i in 0..count {
        if !flicker(motion, 0.8, 5000 + i as u64) {
            continue;
        }
        let x = frame.rect.left() + rnd01(motion, 40.0, i + 300) * frame.rect.width();
        let y = frame.rect.top() + rnd01(motion, 40.0, i + 400) * frame.rect.height();
        let color = pick_rgb(i);
        fill(
            painter,
            x,
            y,
            1.0,
            1.0,
            with_alpha(color, motion.glitch * 200.0),
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

// ─── Label — text color over the face ──────────────────────────────

fn draw_label(painter: &Painter, frame: &Frame, motion: &Motion, text: Color32, label: &str) {
    let active = motion.hover.max(motion.press);
    let text_color = lerp(text, Color32::WHITE, active * 0.55);
    let font = FontId::monospace((15.0 * frame.scale).max(8.0));

    let jitter = if motion.glitch > 0.01 {
        rnd(motion, 50.0, 0) * 10.0 * motion.glitch * frame.scale
    } else {
        0.0
    };
    let pos = frame.rect.center() + vec2(jitter, 0.0);

    // Chromatic ghosts appear ONLY on glitch, not on idle hover.
    if motion.glitch > 0.01 {
        let aberr = (1.0 + motion.glitch * 4.0) * frame.scale;
        let ghost_alpha = 200.0 * motion.glitch;
        painter.text(
            pos - vec2(aberr, 0.0),
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            with_alpha(RGB_R, ghost_alpha),
        );
        painter.text(
            pos + vec2(aberr, 0.0),
            Align2::CENTER_CENTER,
            label,
            font.clone(),
            with_alpha(GHOST_CYAN, ghost_alpha),
        );
    }

    // Soft halo in the text color (so it reads on any face).
    if active > 0.01 || motion.glitch > 0.01 {
        let alpha = (100.0 * active + 150.0 * motion.glitch).min(255.0);
        let glow_color = with_alpha(text, alpha);
        let d = (1.5 * frame.scale).max(1.0);
        for off in [vec2(d, 0.0), vec2(-d, 0.0), vec2(0.0, d), vec2(0.0, -d)] {
            painter.text(
                pos + off,
                Align2::CENTER_CENTER,
                label,
                font.clone(),
                glow_color,
            );
        }
    }

    painter.text(pos, Align2::CENTER_CENTER, label, font, text_color);
}

// ─── Scanlines — only during glitch ───────────────────────────────

fn draw_scanlines(painter: &Painter, frame: &Frame, motion: &Motion) {
    let base = (frame.rect.height() / 14.0).clamp(2.0, 6.0);
    let count = ((base * motion.glitch).ceil() as usize).max(1);

    let speed = 70.0 + motion.glitch as f64 * 200.0;
    let step = frame.rect.height() / count as f32;
    let alpha = (180.0 * motion.glitch).min(255.0);
    let width = (1.0 + motion.glitch * 1.0) * frame.scale.max(0.5);

    for i in 0..count {
        let base_y = frame.rect.top()
            + ((motion.time * speed + i as f64 * step as f64) as f32)
                .rem_euclid(frame.rect.height());
        let jy = rnd(motion, 60.0, i) * 18.0 * motion.glitch * frame.scale;
        let jx = rnd(motion, 40.0, i + 10) * 20.0 * motion.glitch * frame.scale;
        painter.line_segment(
            [
                pos2(frame.rect.left() + 4.0 + jx, base_y + jy),
                pos2(frame.rect.right() - 4.0 + jx, base_y + jy),
            ],
            Stroke::new(width, with_alpha(Color32::WHITE, alpha)),
        );
    }
}

// ─── Glitch intensity ─────────────────────────────────────────────

fn glitch_intensity(ui: &Ui, response: &Response, time: f64) -> f32 {
    let (was_hovered, was_pressed) = ui.ctx().data(|d| {
        (
            d.get_temp::<bool>(response.id.with("wh")).unwrap_or(false),
            d.get_temp::<bool>(response.id.with("wp")).unwrap_or(false),
        )
    });
    let hover_begin = response.hovered() && !was_hovered;
    let press_begin = response.is_pointer_button_down_on() && !was_pressed;
    ui.ctx().data_mut(|d| {
        d.insert_temp(response.id.with("wh"), response.hovered());
        d.insert_temp(response.id.with("wp"), response.is_pointer_button_down_on());
    });

    let key = response.id.with("glitch_at");
    if hover_begin || press_begin {
        ui.ctx().data_mut(|d| d.insert_temp(key, time));
    }
    let last = ui.ctx().data(|d| d.get_temp::<f64>(key)).unwrap_or(-1000.0);
    let t = ((time - last) / GLITCH_DURATION).clamp(0.0, 1.0) as f32;
    if t < 1.0 { (1.0 - t).powi(2) } else { 0.0 }
}

// ─── Helpers ──────────────────────────────────────────────────────

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

fn fill(painter: &Painter, x: f32, y: f32, w: f32, h: f32, color: Color32) {
    painter.rect_filled(Rect::from_min_size(pos2(x, y), vec2(w, h)), 0.0, color);
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
