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
    let seed = time.to_bits();

    let inner_rect = make_inner_rect(&outer_rect);
    let points = make_border_shape(&inner_rect, 8.0);
    let glitches = make_glitch_pixels(&points, 25.0, seed);
    let lines = make_random_horizontal_lines(&points, 25.0, seed);
    let scan = make_scan_line(&points, time, 48.0);
    let glows = make_glow(&outer_rect, &points, 4);
    let grid = make_grid(&points, 8.0);

    draw_glow(painter, Color32::GREEN, &glows);
    draw_border(painter, Color32::WHITE, points.clone(), 1.0);
    fill_shape(painter, Color32::GREEN, points.clone());
    draw_glitches(painter, Color32::BLACK, glitches);
    drow_lines(painter, Color32::GRAY, lines, 1.0);
    draw_scan_line(painter, Color32::PURPLE, scan, 1.0);
    draw_grid(painter, Color32::RED, grid, 1.0);

    ui.request_repaint();

    response
}

pub fn draw_border(painter: &Painter, color: Color32, points: Vec<Pos2>, width: f32) {
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
}

pub fn fill_shape(painter: &Painter, color: Color32, points: Vec<Pos2>) {
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub fn draw_glitches(painter: &Painter, color: Color32, points: Vec<Pos2>) {
    for p in points {
        painter.rect_filled(
            Rect::from_min_max(p, pos2(p.x + 1.0, p.y + 1.0)),
            0.0,
            color,
        );
    }
}

pub fn drow_lines(painter: &Painter, color: Color32, lines: Vec<(Pos2, Pos2)>, width: f32) {
    let stroke = Stroke::new(width, color);
    for (a, b) in lines {
        painter.line_segment([a, b], stroke);
    }
}

pub fn draw_scan_line(painter: &Painter, color: Color32, line: (Pos2, Pos2), width: f32) {
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

pub fn draw_glow(painter: &Painter, color: Color32, glows: &[Vec<Pos2>]) {
    let n = glows.len();
    if n == 0 {
        return;
    }

    // Alpha ramps outermost→innermost, matching the button's
    // 0.06 → 0.22 profile generalized to N layers. Outermost stays
    // very faint (6/255 ≈ 2%); innermost is faint-but-visible
    // (60/255 ≈ 24%). Anything brighter reads as a solid halo,
    // not a glow.
    const OUTER_ALPHA: f32 = 6.0;
    const INNER_ALPHA: f32 = 60.0;

    for (i, layer) in glows.iter().enumerate() {
        let t = if n > 1 {
            i as f32 / (n - 1) as f32
        } else {
            0.0
        };
        let alpha = OUTER_ALPHA + (INNER_ALPHA - OUTER_ALPHA) * t;
        let c = Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            alpha.clamp(0.0, 255.0) as u8,
        );
        painter.add(Shape::convex_polygon(layer.clone(), c, Stroke::NONE));
    }
}

pub fn make_border_shape(rect: &Rect, corner: f32) -> Vec<Pos2> {
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

pub fn make_inner_rect(outer_rect: &Rect) -> Rect {
    outer_rect.shrink(8.0)
}

pub fn make_random_horizontal_lines(
    shape: &Vec<Pos2>,
    density: f32,
    seed: u64,
) -> Vec<(Pos2, Pos2)> {
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

pub fn make_scan_line(
    shape: &Vec<Pos2>,
    time: f64,
    speed_in_pexels_in_seconds: f32,
) -> (Pos2, Pos2) {
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

pub fn make_glow(outer_rect: &Rect, shape: &Vec<Pos2>, layers_of_glow: u8) -> Vec<Vec<Pos2>> {
    if shape.len() < 3 || layers_of_glow == 0 {
        return Vec::new();
    }

    let center = outer_rect.center();
    let short = outer_rect.width().min(outer_rect.height());
    if short <= 0.0 {
        return Vec::new();
    }

    // Normalize the "expand by k px" scaling: distance from center to
    // the farthest vertex. Scaling by `1 + k / max_dist` moves every
    // vertex outward by roughly k px along its radial direction, so
    // the visual ring width matches the requested px amount.
    let max_dist = shape
        .iter()
        .map(|p| (*p - center).length())
        .fold(0.0_f32, f32::max)
        .max(1.0);

    // Match the button's glow profile:
    //   outermost  ≈ 30% of short, capped at 12 px
    //   innermost  ≈ 3 px (never thinner than this)
    // Linear interpolation between the two across N layers.
    const MAX_EXPAND_PX: f32 = 12.0;
    const MIN_EXPAND_PX: f32 = 3.0;
    let outer_px = (short * 0.30).min(MAX_EXPAND_PX);
    let inner_px = MIN_EXPAND_PX.min(outer_px);

    let n = layers_of_glow as usize;
    let mut layers = Vec::with_capacity(n);
    for i in 0..n {
        // i = 0 → outermost (largest, dimmest)
        // i = n-1 → innermost (smallest, brightest)
        let t = if n > 1 {
            i as f32 / (n - 1) as f32
        } else {
            0.0
        };
        let expand = outer_px + (inner_px - outer_px) * t;
        let scale = 1.0 + expand / max_dist;
        layers.push(
            shape
                .iter()
                .map(|p| center + (*p - center) * scale)
                .collect(),
        );
    }
    layers
}

fn polygon_span_at_y(shape: &[Pos2], y: f32) -> Option<(f32, f32)> {
    let n = shape.len();
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for i in 0..n {
        let a = shape[i];
        let b = shape[(i + 1) % n];
        if (a.y - b.y).abs() < f32::EPSILON {
            continue;
        }
        let (ey_lo, ey_hi) = if a.y < b.y { (a.y, b.y) } else { (b.y, a.y) };
        if y < ey_lo || y > ey_hi {
            continue;
        }
        let t = (y - a.y) / (b.y - a.y);
        let x = a.x + t * (b.x - a.x);
        lo = lo.min(x);
        hi = hi.max(x);
    }
    if lo.is_finite() && hi > lo {
        Some((lo, hi))
    } else {
        None
    }
}

fn polygon_span_at_x(shape: &[Pos2], x: f32) -> Option<(f32, f32)> {
    let n = shape.len();
    let mut lo = f32::INFINITY;
    let mut hi = f32::NEG_INFINITY;
    for i in 0..n {
        let a = shape[i];
        let b = shape[(i + 1) % n];
        if (a.x - b.x).abs() < f32::EPSILON {
            continue;
        }
        let (ex_lo, ex_hi) = if a.x < b.x { (a.x, b.x) } else { (b.x, a.x) };
        if x < ex_lo || x > ex_hi {
            continue;
        }
        let t = (x - a.x) / (b.x - a.x);
        let y = a.y + t * (b.y - a.y);
        lo = lo.min(y);
        hi = hi.max(y);
    }
    if lo.is_finite() && hi > lo {
        Some((lo, hi))
    } else {
        None
    }
}

fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn make_grid(
    shape: &Vec<Pos2>,
    square_size_in_pixels: f32,
) -> (Vec<(Pos2, Pos2)>, Vec<(Pos2, Pos2)>) {
    let mut rows: Vec<(Pos2, Pos2)> = Vec::new();
    let mut cols: Vec<(Pos2, Pos2)> = Vec::new();

    if shape.len() < 3 || square_size_in_pixels <= 0.0 || !square_size_in_pixels.is_finite() {
        return (rows, cols);
    }

    // ── Bounding box ───────────────────────────────────────────
    let mut x_min = f32::INFINITY;
    let mut x_max = f32::NEG_INFINITY;
    let mut y_min = f32::INFINITY;
    let mut y_max = f32::NEG_INFINITY;
    for p in shape {
        x_min = x_min.min(p.x);
        x_max = x_max.max(p.x);
        y_min = y_min.min(p.y);
        y_max = y_max.max(p.y);
    }
    let w = x_max - x_min;
    let h = y_max - y_min;
    if w <= 0.0 || h <= 0.0 {
        return (rows, cols);
    }

    // ── Derive cell_size so cells are truly square + uniform ──
    // Preferred: a divisor of GCD(round(w), round(h)) close to
    // `square_size_in_pixels`. Since it divides both axes exactly,
    // every cell ends up the same size — no partial edge cell.
    // Fallback (coprime dimensions): divide the *shorter* axis
    // into round(short / target) equal parts, which keeps the
    // shorter axis uniform at the cost of one partial cell on the
    // longer axis.
    let w_i = w.round().max(1.0) as usize;
    let h_i = h.round().max(1.0) as usize;
    let g = gcd(w_i, h_i);
    let target = square_size_in_pixels.round().max(1.0) as usize;

    let cell_size = if g >= 3 {
        // Closest divisor of g to `target`.
        let mut best = 1usize;
        let mut best_diff = usize::MAX;
        for d in 1..=g {
            if g % d != 0 {
                continue;
            }
            let diff = d.abs_diff(target);
            if diff < best_diff {
                best_diff = diff;
                best = d;
            }
        }
        // Reject divisors more than 2× off target — at that point
        // the cell is visibly the wrong size. Fall back instead.
        if best * 2 >= target && best <= target * 2 {
            best as f32
        } else {
            fallback_cell_size(w, h, square_size_in_pixels)
        }
    } else {
        fallback_cell_size(w, h, square_size_in_pixels)
    };

    let n_rows = (h / cell_size).floor() as usize;
    let n_cols = (w / cell_size).floor() as usize;

    // Skip i = 0 (first line sits on the border) and i = n (same
    // on the far side). Only interior lines get emitted.
    for i in 1..n_rows {
        let y = y_min + i as f32 * cell_size;
        if let Some((x0, x1)) = polygon_span_at_y(shape, y) {
            rows.push((pos2(x0, y), pos2(x1, y)));
        }
    }
    for i in 1..n_cols {
        let x = x_min + i as f32 * cell_size;
        if let Some((y0, y1)) = polygon_span_at_x(shape, x) {
            cols.push((pos2(x, y0), pos2(x, y1)));
        }
    }

    (rows, cols)
}

fn fallback_cell_size(w: f32, h: f32, target: f32) -> f32 {
    // Uniform along the shorter axis; longer axis gets one partial
    // cell at the far end (suppressed by `1..n_rows` above).
    let min_dim = w.min(h);
    let divisions = (min_dim / target).round().max(1.0);
    min_dim / divisions
}

fn draw_grid(
    painter: &Painter,
    color: Color32,
    grid: (Vec<(Pos2, Pos2)>, Vec<(Pos2, Pos2)>),
    width: f32,
) {
    let (rows, cols) = grid;
    if width <= 0.0 || !width.is_finite() {
        return;
    }

    // ── Axis centres from first/last line ──────────────────────
    let (row_center, row_half) = if let (Some(f), Some(l)) = (rows.first(), rows.last()) {
        let lo = f.0.y.min(l.0.y);
        let hi = f.0.y.max(l.0.y);
        ((lo + hi) * 0.5, ((hi - lo) * 0.5).max(f32::EPSILON))
    } else {
        (0.0, 1.0)
    };
    let (col_center, col_half) = if let (Some(f), Some(l)) = (cols.first(), cols.last()) {
        let lo = f.0.x.min(l.0.x);
        let hi = f.0.x.max(l.0.x);
        ((lo + hi) * 0.5, ((hi - lo) * 0.5).max(f32::EPSILON))
    } else {
        (0.0, 1.0)
    };

    // ── Gradual alpha fade-out toward the edges ───────────────
    // Centre = color.a() exactly (as passed).
    // Edge   = EDGE_FACTOR × color.a()  → more transparent.
    // t goes 0 (centre) → 1 (edge), so the interpolation is a
    // continuous ramp, not a two-band step.
    const EDGE_FACTOR: f32 = 0.15;
    let a_center = color.a() as f32;
    let a_edge = a_center * EDGE_FACTOR;
    let alpha_at = |t: f32| -> u8 {
        let t = t.clamp(0.0, 1.0);
        (a_center + (a_edge - a_center) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };

    for (a, b) in rows {
        let t = (a.y - row_center).abs() / row_half;
        let c = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha_at(t));
        painter.line_segment([a, b], Stroke::new(width, c));
    }
    for (a, b) in cols {
        let t = (a.x - col_center).abs() / col_half;
        let c = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha_at(t));
        painter.line_segment([a, b], Stroke::new(width, c));
    }
}
