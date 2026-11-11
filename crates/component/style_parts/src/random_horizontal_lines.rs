use prelude::egui::Color32;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Stroke;
use prelude::egui::pos2;

pub fn drow(painter: &Painter, color: Color32, lines: Vec<(Pos2, Pos2)>, width: f32) {
    let stroke = Stroke::new(width, color);
    for (a, b) in lines {
        painter.line_segment([a, b], stroke);
    }
}

pub fn make(shape: &Vec<Pos2>, density: f32, seed: u64) -> Vec<(Pos2, Pos2)> {
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
