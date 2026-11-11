use prelude::egui::Color32;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Stroke;
use prelude::egui::pos2;

pub fn draw(painter: &Painter, color: Color32, line: (Pos2, Pos2), width: f32) {
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

pub fn make(shape: &Vec<Pos2>, time: f64, speed_in_pexels_in_seconds: f32) -> (Pos2, Pos2) {
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
