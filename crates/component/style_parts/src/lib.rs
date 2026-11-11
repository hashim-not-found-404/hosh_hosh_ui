use prelude::egui::Color32;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Shape;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::egui::pos2;
mod glitches;

use glitches::make_glitch_pixels;

pub fn render(ui: &mut Ui) -> Response {
    let (outer_rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

    if outer_rect.width() <= 0.0 || outer_rect.height() <= 0.0 {
        return response;
    }

    let painter = ui.painter();
    let time = ui.input(|i| i.time);

    let inner_rect = make_inner_rect(&outer_rect);
    let points = make_border_shape(&inner_rect, 8.0);
    let glitches = make_glitch_pixels(&points, 25.0, time as u64);
    let lines = make_random_horizontal_lines(&points, 25.0, time as u64);
    let scan = make_scan_line(&points, time, 48.0);

    draw_border(painter, Color32::WHITE, points.clone(), 1.0);
    fill_shape(painter, Color32::GREEN, points.clone());
    draw_glitches(painter, Color32::BLACK, glitches);
    drow_lines(painter, Color32::GRAY, lines, 1.0);
    draw_scan_line(painter, Color32::PURPLE, scan, 1.0);

    ui.request_repaint();

    response
}

fn draw_border(painter: &Painter, color: Color32, points: Vec<Pos2>, width: f32) {
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
}

fn fill_shape(painter: &Painter, color: Color32, points: Vec<Pos2>) {
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

fn draw_glitches(painter: &Painter, color: Color32, points: Vec<Pos2>) {
    for p in points {
        painter.rect_filled(
            Rect::from_min_max(p, pos2(p.x + 1.0, p.y + 1.0)),
            0.0,
            color,
        );
    }
}

fn drow_lines(painter: &Painter, color: Color32, lines: Vec<(Pos2, Pos2)>, width: f32) {
    let stroke = Stroke::new(width, color);
    for (a, b) in lines {
        painter.line_segment([a, b], stroke);
    }
}

fn make_border_shape(rect: &Rect, corner: f32) -> Vec<Pos2> {
    let corner = corner
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);

    vec![
        pos2(rect.left(), rect.top()),
        pos2(rect.right() - corner, rect.top()),
        pos2(rect.right(), rect.top() + corner),
        pos2(rect.right(), rect.bottom()),
        pos2(rect.right() - corner, rect.bottom()),
        pos2(rect.left() + corner, rect.bottom()),
        pos2(rect.left(), rect.bottom() - corner),
        pos2(rect.left(), rect.top() + corner),
    ]
}

fn make_inner_rect(outer_rect: &Rect) -> Rect {
    outer_rect.shrink(8.0)
}

fn make_random_horizontal_lines(shape: &Vec<Pos2>, density: f32, seed: u64) -> Vec<(Pos2, Pos2)> {
    if shape.len() < 3 || density <= 0.0 || !density.is_finite() {
        return Vec::new();
    }

    // ── 1) Vertical extent ─────────────────────────────────────
    let mut y_min = f32::INFINITY;
    let mut y_max = f32::NEG_INFINITY;
    for p in shape {
        y_min = y_min.min(p.y);
        y_max = y_max.max(p.y);
    }
    let height = y_max - y_min;
    if height <= 0.0 {
        return Vec::new();
    }

    let target = (height / density).round() as usize;
    if target == 0 {
        return Vec::new();
    }

    // ── 2) Precompute edges once ───────────────────────────────
    // Each edge as (y_lo, y_hi, slope, offset), with
    //   x(y) = slope * y + offset
    // Horizontal and degenerate edges are dropped here so the
    // per-line loop never re-tests or re-divides them.
    let n = shape.len();
    let mut edges: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(n);
    for i in 0..n {
        let a = shape[i];
        let b = shape[(i + 1) % n];
        if (a.y - b.y).abs() < f32::EPSILON {
            continue;
        }
        let (lo, hi, x_lo, x_hi) = if a.y < b.y {
            (a.y, b.y, a.x, b.x)
        } else {
            (b.y, a.y, b.x, a.x)
        };
        let slope = (x_hi - x_lo) / (hi - lo);
        let offset = x_lo - slope * lo;
        edges.push((lo, hi, slope, offset));
    }
    if edges.is_empty() {
        return Vec::new();
    }

    // ── 3) Deterministic RNG: xorshift64, same as glitches ─────
    let mut state = seed | 1;
    let mut next = move || -> f32 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 40) as f32 * (1.0 / 16_777_216.0) // [0, 1)
    };

    // ── 4) Reused crossing buffer — no per-line allocation ─────
    let mut xs: Vec<f32> = Vec::with_capacity(edges.len());
    let mut out: Vec<(Pos2, Pos2)> = Vec::with_capacity(target);

    for _ in 0..target {
        let y = y_min + next() * height;

        // ── Collect crossings ──────────────────────────────────
        // Half-open [lo, hi) so a shared vertex counts once;
        // horizontal edges are already gone from `edges`.
        xs.clear();
        for &(lo, hi, slope, offset) in &edges {
            if y < lo || y >= hi {
                continue;
            }
            xs.push(slope.mul_add(y, offset));
        }

        if xs.len() < 2 {
            continue;
        }
        // total_cmp is branchless (no NaN dance, no Option).
        xs.sort_unstable_by(f32::total_cmp);

        // ── Pair crossings → inside spans, then vary ───────────
        let mut i = 0;
        while i + 1 < xs.len() {
            let x0 = xs[i];
            let x1 = xs[i + 1];
            i += 2;

            let span = x1 - x0;
            if span <= 0.5 {
                continue;
            }

            // roll < 0.35 → full width; otherwise trim both ends
            // independently, biased small via powi(2).
            let roll = next();
            let (lx, rx) = if roll < 0.35 {
                (x0, x1)
            } else {
                let min_keep = 0.20;
                let mut a = next().powi(2) * (1.0 - min_keep);
                let mut b = next().powi(2) * (1.0 - min_keep);
                if a + b > 1.0 - min_keep {
                    let s = (1.0 - min_keep) / (a + b);
                    a *= s;
                    b *= s;
                }
                (x0 + a * span, x1 - b * span)
            };

            if rx - lx > 0.5 {
                out.push((pos2(lx, y), pos2(rx, y)));
            }
        }
    }

    out
}

fn make_scan_line(shape: &Vec<Pos2>, time: f64, speed_in_pexels_in_seconds: f32) -> (Pos2, Pos2) {
    let time = time as f32;

    if shape.len() < 3 {
        return (Pos2::ZERO, Pos2::ZERO);
    }

    // ── 1) Vertical extent ─────────────────────────────────────
    let mut y_min = f32::INFINITY;
    let mut y_max = f32::NEG_INFINITY;
    for p in shape {
        y_min = y_min.min(p.y);
        y_max = y_max.max(p.y);
    }
    let height = y_max - y_min;
    if height <= 0.0 || !speed_in_pexels_in_seconds.is_finite() || speed_in_pexels_in_seconds <= 0.0
    {
        return (Pos2::ZERO, Pos2::ZERO);
    }

    // ── 2) Triangle-wave ping-pong at `speed` px/s ────────────
    // No inset: phase 0 lands on y_min (top border) and phase 1
    // lands on y_max (bottom border) — the turnarounds touch.
    let one_way = height / speed_in_pexels_in_seconds;
    let cycle = 2.0 * one_way;

    let t = time.rem_euclid(cycle);
    let phase = if t < one_way {
        t / one_way // descending half — top → bottom
    } else {
        (cycle - t) / one_way // ascending half — bottom → top
    };

    let y = y_min + phase * height;

    // ── 3) Scanline ∩ polygon → min/max x at this y ────────────
    let n = shape.len();
    let mut x_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    for i in 0..n {
        let a = shape[i];
        let b = shape[(i + 1) % n];

        // Horizontal edge: only matters if we're exactly on it.
        if (a.y - b.y).abs() < f32::EPSILON {
            if (y - a.y).abs() < f32::EPSILON {
                x_min = x_min.min(a.x).min(b.x);
                x_max = x_max.max(a.x).max(b.x);
            }
            continue;
        }

        let (lo, hi) = if a.y < b.y { (a.y, b.y) } else { (b.y, a.y) };
        if y < lo || y > hi {
            continue;
        }
        let t = (y - a.y) / (b.y - a.y);
        let x = a.x + t * (b.x - a.x);
        if x < x_min {
            x_min = x;
        }
        if x > x_max {
            x_max = x;
        }
    }

    if !x_min.is_finite() || !x_max.is_finite() || x_max <= x_min {
        return (Pos2::ZERO, Pos2::ZERO);
    }

    // No horizontal pad either — endpoints coincide with the
    // polygon outline at this y, so they touch the left/right
    // borders too (including where the cut corners narrow it).
    (pos2(x_min, y), pos2(x_max, y))
}

fn draw_scan_line(painter: &Painter, color: Color32, line: (Pos2, Pos2), width: f32) {
    let (a, b) = line;

    // Degenerate (make_scan_line bailed out) → nothing to draw.
    if (a.x - b.x).abs() < 0.5 && (a.y - b.y).abs() < 0.5 {
        return;
    }
    if width <= 0.0 || !width.is_finite() {
        return;
    }

    painter.line_segment([a, b], Stroke::new(width, color));
}
