use prelude::PrimaryButton;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
use prelude::egui::Pos2;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Shape;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

const FACE_PAD: f32 = 16.0;
const CORNER_CUT: f32 = 10.0;
const GLITCH_DURATION: f64 = 0.30;

pub struct DefaultPrimaryButton;

impl PrimaryButton for DefaultPrimaryButton {
    fn render(ui: &mut Ui, t: &dyn Theme, label: &str) -> Response {
        button(ui, t, label)
    }
}

fn button(ui: &mut Ui, t: &dyn Theme, label: &str) -> Response {
    let desired = ui.available_size();
    let (outer, response) = ui.allocate_exact_size(desired, Sense::click());
    if outer.width() <= 0.0 || outer.height() <= 0.0 {
        return response;
    }

    // Padding and corners scale with size — no minimum threshold.
    let pad = FACE_PAD
        .min(outer.width() * 0.20)
        .min(outer.height() * 0.20);
    let face = outer.shrink(pad);
    let short = face.width().min(face.height());
    let corner = CORNER_CUT.min(short * 0.28).max(0.0);
    let scale = (face.height() / 40.0).clamp(0.3, 1.0);

    let time = ui.input(|i| i.time);
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id.with("h"), response.hovered(), 0.16);
    let press = ui.ctx().animate_bool_with_time(
        response.id.with("p"),
        response.is_pointer_button_down_on(),
        0.05,
    );

    // Glitch triggers on *transition into* hover or press.
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

    let glitch_key = response.id.with("glitch_at");
    if hover_begin || press_begin {
        ui.ctx().data_mut(|d| d.insert_temp(glitch_key, time));
    }
    let last = ui
        .ctx()
        .data(|d| d.get_temp::<f64>(glitch_key))
        .unwrap_or(-1000.0);
    let gt = ((time - last) / GLITCH_DURATION).clamp(0.0, 1.0) as f32;
    let glitch = if gt < 1.0 { (1.0 - gt).powi(2) } else { 0.0 };

    let primary = t.primary();
    let on_primary = t.on_primary();
    let painter = ui.painter().with_clip_rect(outer);
    let red = Color32::from_rgb(255, 40, 100);
    let cyan = Color32::from_rgb(40, 220, 255);
    let white = Color32::WHITE;

    // ── 1. Breathing glow ────────────────────────────────────────
    let pulse = ((time * 2.5).sin() as f32 * 0.5 + 0.5) * hover;
    let glow = 0.35 + 0.65 * hover + 0.15 * pulse + 0.4 * glitch;
    let e1 = (short * 0.35).min(14.0);
    let e2 = (short * 0.20).min(8.0);
    let e3 = (short * 0.10).min(4.0);
    for (expand, mult) in [(e1, 0.06_f32), (e2, 0.14), (e3, 0.24)] {
        let alpha = (mult * glow * 255.0).min(255.0) as u8;
        painter.add(Shape::convex_polygon(
            cut_polygon(face.expand(expand), corner + expand),
            Color32::from_rgba_unmultiplied(primary.r(), primary.g(), primary.b(), alpha),
            Stroke::NONE,
        ));
    }

    // ── 2. Face ──────────────────────────────────────────────────
    let poly = cut_polygon(face, corner);
    painter.add(Shape::convex_polygon(
        poly.clone(),
        t.surface_alt(),
        Stroke::NONE,
    ));

    // ── 3. Hover tint ────────────────────────────────────────────
    if hover > 0.01 {
        painter.add(Shape::convex_polygon(
            poly.clone(),
            Color32::from_rgba_unmultiplied(
                primary.r(),
                primary.g(),
                primary.b(),
                (44.0 * hover) as u8,
            ),
            Stroke::NONE,
        ));
    }

    // ── 4. Warning tape (size-safe) ─────────────────────────────
    let accent = Color32::from_rgba_unmultiplied(
        primary.r(),
        primary.g(),
        primary.b(),
        (180.0 + 75.0 * hover) as u8,
    );
    let tape_stroke = Stroke::new(2.0, accent);
    let tape_len = (short * 0.35).min(CORNER_CUT + 10.0).max(2.0);
    let tape_step = (tape_len * 0.2).max(1.0);
    let tape_inset = (tape_len * 0.15).max(1.5);
    for i in 0..3 {
        let off = i as f32 * tape_step;
        if tape_len - off < 1.5 {
            break;
        }
        painter.line_segment(
            [
                pos2(
                    face.left() + tape_inset + off,
                    face.top() + tape_inset + tape_len - off,
                ),
                pos2(
                    face.left() + tape_inset + tape_len - off,
                    face.top() + tape_inset + off,
                ),
            ],
            tape_stroke,
        );
        painter.line_segment(
            [
                pos2(
                    face.right() - tape_inset - off,
                    face.bottom() - tape_inset - tape_len + off,
                ),
                pos2(
                    face.right() - tape_inset - tape_len + off,
                    face.bottom() - tape_inset - off,
                ),
            ],
            tape_stroke,
        );
    }

    // ── 5. Border ────────────────────────────────────────────────
    let border_w = 1.5 + hover * 1.5 + glitch * 2.5;
    let border_a = (130.0 + 125.0 * hover).min(255.0) as u8;
    painter.add(Shape::closed_line(
        poly.clone(),
        Stroke::new(
            border_w,
            Color32::from_rgba_unmultiplied(primary.r(), primary.g(), primary.b(), border_a),
        ),
    ));

    // ── 6. Glitch — 4 independent types, all size-scaled ────────
    if glitch > 0.01 {
        let area = face.width() * face.height();
        let base = ((area / 1500.0) as usize).clamp(3, 24);

        // (a) Horizontal tear bands
        let tear_count = (base / 2).max(2);
        for i in 0..tear_count {
            if !flicker(glitch, time, 10 + i as u64) {
                continue;
            }
            let y =
                face.top() + (noise((time * 25.0) as u64, i) + 0.5).clamp(0.0, 1.0) * face.height();
            let h =
                1.0 + (noise((time * 25.0) as u64, i + 100) + 0.5).clamp(0.0, 1.0) * (4.0 * scale);
            let shift = noise((time * 25.0) as u64, i + 200) * 20.0 * glitch * scale;
            let color = match i % 3 {
                0 => red,
                1 => cyan,
                _ => white,
            };
            painter.rect_filled(
                Rect::from_min_size(pos2(face.left() + shift, y), vec2(face.width(), h)),
                0.0,
                Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    (glitch * 200.0) as u8,
                ),
            );
        }

        // (b) Pixel debris — scattered single pixels
        let pixel_count = base;
        for i in 0..pixel_count {
            if !flicker(glitch, time, 500 + i as u64) {
                continue;
            }
            let px = face.left()
                + (noise((time * 40.0) as u64, i + 300) + 0.5).clamp(0.0, 1.0) * face.width();
            let py = face.top()
                + (noise((time * 40.0) as u64, i + 400) + 0.5).clamp(0.0, 1.0) * face.height();
            let w = (1.0
                + (noise((time * 40.0) as u64, i + 500) + 0.5).clamp(0.0, 1.0) * (3.0 * scale))
                .max(1.0);
            let h = (1.0
                + (noise((time * 40.0) as u64, i + 600) + 0.5).clamp(0.0, 1.0) * (2.0 * scale))
                .max(1.0);
            let color = match i % 4 {
                0 => red,
                1 => cyan,
                2 => white,
                _ => primary,
            };
            painter.rect_filled(
                Rect::from_min_size(pos2(px, py), vec2(w, h)),
                0.0,
                Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    (glitch * 220.0) as u8,
                ),
            );
        }

        // (c) Damage cluster — a burst of nearby pixels (simulates hot-pixel blob)
        if flicker(glitch * 0.7, time, 777) {
            let cx = face.left()
                + (noise((time * 20.0) as u64, 900) + 0.5).clamp(0.0, 1.0) * face.width();
            let cy = face.top()
                + (noise((time * 20.0) as u64, 910) + 0.5).clamp(0.0, 1.0) * face.height();
            let blob =
                3 + ((noise((time * 20.0) as u64, 920) + 0.5).clamp(0.0, 1.0) * 6.0) as usize;
            let color = if noise((time * 20.0) as u64, 930) > 0.0 {
                red
            } else {
                cyan
            };
            for j in 0..blob {
                let dx = noise((time * 20.0) as u64, 940 + j) * (8.0 * scale);
                let dy = noise((time * 20.0) as u64, 950 + j) * (6.0 * scale);
                let s = (1.0
                    + (noise((time * 20.0) as u64, 960 + j) + 0.5).clamp(0.0, 1.0) * (2.0 * scale))
                    .max(1.0);
                painter.rect_filled(
                    Rect::from_min_size(pos2(cx + dx, cy + dy), vec2(s, s)),
                    0.0,
                    Color32::from_rgba_unmultiplied(
                        color.r(),
                        color.g(),
                        color.b(),
                        (glitch * 220.0) as u8,
                    ),
                );
            }
        }

        // (d) Vertical stripe — rarer, larger disruption
        if flicker(glitch * 0.4, time, 888) {
            let x = face.left()
                + (noise((time * 15.0) as u64, 800) + 0.5).clamp(0.0, 1.0) * face.width();
            let w = (2.0
                + (noise((time * 15.0) as u64, 810) + 0.5).clamp(0.0, 1.0) * (6.0 * scale))
                .max(1.0);
            let color = if noise((time * 15.0) as u64, 820) > 0.0 {
                red
            } else {
                cyan
            };
            painter.rect_filled(
                Rect::from_min_size(pos2(x, face.top()), vec2(w, face.height())),
                0.0,
                Color32::from_rgba_unmultiplied(
                    color.r(),
                    color.g(),
                    color.b(),
                    (glitch * 120.0) as u8,
                ),
            );
        }
    }

    // ── 7. Chromatic label ──────────────────────────────────────
    let text_color = lerp(primary, on_primary, hover.max(press));
    let font_size = (16.0 * scale).max(8.0);
    let font = FontId::monospace(font_size);
    let aberr = (1.0 + hover * 1.0 + glitch * 6.0) * scale;
    let jitter = if glitch > 0.01 {
        noise((time * 50.0) as u64, 0) * 10.0 * glitch * scale
    } else {
        0.0
    };
    let label_pos = face.center() + vec2(jitter, 0.0);

    let ghost_a = (170.0 + 85.0 * hover.max(glitch)).min(255.0) as u8;
    painter.text(
        label_pos + vec2(-aberr, 0.0),
        Align2::CENTER_CENTER,
        label,
        font.clone(),
        Color32::from_rgba_unmultiplied(red.r(), red.g(), red.b(), ghost_a),
    );
    painter.text(
        label_pos + vec2(aberr, 0.0),
        Align2::CENTER_CENTER,
        label,
        font.clone(),
        Color32::from_rgba_unmultiplied(cyan.r(), cyan.g(), cyan.b(), ghost_a),
    );

    if hover > 0.01 || glitch > 0.01 {
        let ga = (110.0 * hover + 180.0 * glitch).min(255.0) as u8;
        let gc = Color32::from_rgba_unmultiplied(primary.r(), primary.g(), primary.b(), ga);
        let glow_d = (2.0 * scale).max(1.0);
        for off in [
            vec2(glow_d, 0.0),
            vec2(-glow_d, 0.0),
            vec2(0.0, glow_d),
            vec2(0.0, -glow_d),
        ] {
            painter.text(
                label_pos + off,
                Align2::CENTER_CENTER,
                label,
                font.clone(),
                gc,
            );
        }
    }
    painter.text(label_pos, Align2::CENTER_CENTER, label, font, text_color);

    // ── 8. Scanlines — count scales with size AND glitch ────────
    let scan_count = if glitch > 0.01 {
        let base = (face.height() / 12.0).clamp(2.0, 8.0);
        ((base * glitch).ceil() as usize).max(1)
    } else if hover > 0.01 {
        1
    } else {
        0
    };
    if scan_count > 0 {
        let speed = 60.0 + glitch as f64 * 240.0;
        for i in 0..scan_count {
            let offset = (i as f64) * (face.height() as f64 / scan_count as f64);
            let base_y = face.top() + ((time * speed + offset) as f32).rem_euclid(face.height());
            let jy = if glitch > 0.01 {
                noise((time * 60.0) as u64, i as usize) * 18.0 * glitch * scale
            } else {
                0.0
            };
            let jx = if glitch > 0.01 {
                noise((time * 40.0) as u64, i as usize + 10) * 20.0 * glitch * scale
            } else {
                0.0
            };
            let alpha = (110.0 * hover + 220.0 * glitch).min(255.0) as u8;
            painter.line_segment(
                [
                    pos2(face.left() + 4.0 + jx, base_y + jy),
                    pos2(face.right() - 4.0 + jx, base_y + jy),
                ],
                Stroke::new(
                    (1.0 + glitch * 1.5) * scale.max(0.5),
                    Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                ),
            );
        }
    }

    // ── 9. Cursor + repaint ─────────────────────────────────────
    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if response.hovered() || glitch > 0.001 || (0.001..0.999).contains(&hover) {
        ui.ctx().request_repaint();
    }

    response
}

fn flicker(glitch: f32, time: f64, seed: u64) -> bool {
    if glitch <= 0.01 {
        return false;
    }
    let n = noise((time * 30.0) as u64, seed as usize) + 0.5; // 0..1
    n < glitch * 1.2
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
