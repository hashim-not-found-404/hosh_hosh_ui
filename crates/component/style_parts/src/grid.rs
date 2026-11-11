use prelude::egui::Color32;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Stroke;
use prelude::egui::pos2;

pub fn draw(
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

pub fn make(
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

fn fallback_cell_size(w: f32, h: f32, target: f32) -> f32 {
    // Uniform along the shorter axis; longer axis gets one partial
    // cell at the far end (suppressed by `1..n_rows` above).
    let min_dim = w.min(h);
    let divisions = (min_dim / target).round().max(1.0);
    min_dim / divisions
}
