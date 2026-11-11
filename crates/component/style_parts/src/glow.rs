use prelude::egui::Color32;
use prelude::egui::Mesh;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Rect;
use prelude::egui::Shape;
use prelude::egui::Stroke;
use prelude::egui::pos2;

pub fn draw(painter: &Painter, color: Color32, glows: &[Vec<Pos2>]) {
    let n = glows.len();
    if n == 0 {
        return;
    }

    // Alpha ramps outermost→innermost. Outermost ring ≈ 2%,
    // innermost ring ≈ 24% — same profile as before.
    const OUTER_ALPHA: f32 = 6.0;
    const INNER_ALPHA: f32 = 60.0;

    // Single layer: no neighbour to derive an inner boundary from.
    // Fall back to a full polygon; the fill (drawn after) hides the
    // interior, so visually identical.
    if n == 1 {
        painter.add(Shape::convex_polygon(
            glows[0].clone(),
            with_alpha(color, OUTER_ALPHA),
            Stroke::NONE,
        ));
        return;
    }

    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let alpha = OUTER_ALPHA + (INNER_ALPHA - OUTER_ALPHA) * t;
        let c = with_alpha(color, alpha);

        // Outer boundary is glow[i]. Inner boundary is glow[i+1] for
        // all but the last layer; for the innermost, extrapolate the
        // source from the last two layers (exact for n = 4).
        if i + 1 < n {
            add_ring(painter, &glows[i + 1], &glows[i], c);
        } else {
            let source: Vec<Pos2> = glows[n - 1]
                .iter()
                .zip(&glows[n - 2])
                .map(|(outer, inner)| pos2(2.0 * outer.x - inner.x, 2.0 * outer.y - inner.y))
                .collect();
            add_ring(painter, &source, &glows[i], c);
        }
    }
}

/// Emit two triangles per edge, forming the band between `inner`
/// and `outer`. Both slices must have the same vertex count (true:
/// every glow layer is a uniform scale of the same source polygon).
fn add_ring(painter: &Painter, inner: &[Pos2], outer: &[Pos2], color: Color32) {
    let n = inner.len();
    if n < 3 || outer.len() != n {
        return;
    }

    let mut mesh = Mesh::default();
    for &p in inner {
        mesh.colored_vertex(p, color);
    }
    for &p in outer {
        mesh.colored_vertex(p, color);
    }

    let nu = n as u32;
    for i in 0..nu {
        let next = (i + 1) % nu;
        // Quad: inner[i], inner[next], outer[next], outer[i]
        // → 2 triangles, both sharing vertex i (indices 0..n = inner,
        //   n..2n = outer).
        mesh.add_triangle(i, next, nu + next);
        mesh.add_triangle(i, nu + next, nu + i);
    }

    painter.add(Shape::mesh(mesh));
}

fn with_alpha(color: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        color.r(),
        color.g(),
        color.b(),
        alpha.clamp(0.0, 255.0) as u8,
    )
}

// make() — unchanged, same signature, same return type
pub fn make(outer_rect: &Rect, shape: &Vec<Pos2>, layers_of_glow: u8) -> Vec<Vec<Pos2>> {
    if shape.len() < 3 || layers_of_glow == 0 {
        return Vec::new();
    }

    let center = outer_rect.center();
    let short = outer_rect.width().min(outer_rect.height());
    if short <= 0.0 {
        return Vec::new();
    }

    let max_dist = shape
        .iter()
        .map(|p| (*p - center).length())
        .fold(0.0_f32, f32::max)
        .max(1.0);

    const MAX_EXPAND_PX: f32 = 12.0;
    const MIN_EXPAND_PX: f32 = 3.0;
    let outer_px = (short * 0.30).min(MAX_EXPAND_PX);
    let inner_px = MIN_EXPAND_PX.min(outer_px);

    let n = layers_of_glow as usize;
    let mut layers = Vec::with_capacity(n);
    for i in 0..n {
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
