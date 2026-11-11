use prelude::egui::{Pos2, pos2};

pub(crate) fn make_glitch_pixels(shape: Vec<Pos2>, density: f32, seed: u64) -> Vec<Pos2> {
    if shape.len() < 3 || density <= 0.0 || !density.is_finite() {
        return Vec::new();
    }

    // ── 1) Triangulate once — any simple polygon, concave OK ─────
    let triangles = ear_clip(&shape);
    if triangles.is_empty() {
        return Vec::new();
    }

    // ── 2) Cumulative area weights for the triangle pick ─────────
    let mut cumulative = Vec::with_capacity(triangles.len());
    let mut total = 0.0f32;
    for (a, b, c) in &triangles {
        total += 0.5 * ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs();
        cumulative.push(total);
    }
    if total <= 0.0 {
        return Vec::new();
    }

    // ── 3) Target from *exact* area, not bbox ───────────────────
    //      Concave shapes no longer over-generate.
    let target = (total / density).round().max(0.0) as usize;
    if target == 0 {
        return Vec::new();
    }

    // ── 4) Fast deterministic RNG: xorshift64, seeded from `seed` ─
    //      `| 1` avoids the all-zero fixed point.
    let mut state = seed | 1;
    let mut next = move || -> f32 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 40) as f32 * (1.0 / 16_777_216.0) // [0, 1)
    };

    // ── 5) Sample: no retry loop, one shot per pixel ─────────────
    let mut out = Vec::with_capacity(target);
    for _ in 0..target {
        // Area-weighted triangle pick — k is tiny, linear scan wins.
        let r = next() * total;
        let ti = cumulative
            .iter()
            .position(|&c| r <= c)
            .unwrap_or(triangles.len() - 1);

        // Uniform-in-triangle via √-barycentric (no seam).
        let r1 = next();
        let r2 = next();
        let s = r1.sqrt();
        let wa = 1.0 - s;
        let wb = s * (1.0 - r2);
        let wc = s * r2;

        let (a, b, c) = triangles[ti];
        out.push(pos2(
            wa * a.x + wb * b.x + wc * c.x,
            wa * a.y + wb * b.y + wc * c.y,
        ));
    }

    out
}

fn ear_clip(shape: &[Pos2]) -> Vec<(Pos2, Pos2, Pos2)> {
    let n = shape.len();
    if n < 3 {
        return Vec::new();
    }

    let ccw = polygon_area_signed(shape) > 0.0;
    let mut indices: Vec<usize> = (0..n).collect();
    let mut triangles = Vec::with_capacity(n.saturating_sub(2));

    while indices.len() > 3 {
        let m = indices.len();
        let mut clipped = false;

        for i in 0..m {
            let prev = indices[(i + m - 1) % m];
            let curr = indices[i];
            let next = indices[(i + 1) % m];

            let a = shape[prev];
            let b = shape[curr];
            let c = shape[next];

            let cross = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);
            let is_convex = if ccw { cross > 0.0 } else { cross < 0.0 };
            if !is_convex {
                continue;
            }

            let mut blocked = false;
            for &other in &indices {
                if other == prev || other == curr || other == next {
                    continue;
                }
                if point_in_triangle(shape[other], a, b, c) {
                    blocked = true;
                    break;
                }
            }
            if blocked {
                continue;
            }

            triangles.push((a, b, c));
            indices.remove(i);
            clipped = true;
            break;
        }

        if !clipped {
            break;
        }
    }

    if indices.len() == 3 {
        triangles.push((shape[indices[0]], shape[indices[1]], shape[indices[2]]));
    }

    triangles
}

fn polygon_area_signed(shape: &[Pos2]) -> f32 {
    let n = shape.len();
    let mut a = 0.0f32;
    for i in 0..n {
        let p0 = shape[i];
        let p1 = shape[(i + 1) % n];
        a += p0.x * p1.y - p1.x * p0.y;
    }
    a * 0.5
}

fn point_in_triangle(p: Pos2, a: Pos2, b: Pos2, c: Pos2) -> bool {
    let d1 = (p.x - b.x) * (a.y - b.y) - (a.x - b.x) * (p.y - b.y);
    let d2 = (p.x - c.x) * (b.y - c.y) - (b.x - c.x) * (p.y - c.y);
    let d3 = (p.x - a.x) * (c.y - a.y) - (c.x - a.x) * (p.y - a.y);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}
