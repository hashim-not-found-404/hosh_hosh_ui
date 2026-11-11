use prelude::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke};

pub fn draw(painter: &Painter, color: Color32, glows: &[Vec<Pos2>]) {
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

pub fn make(outer_rect: &Rect, shape: &Vec<Pos2>, layers_of_glow: u8) -> Vec<Vec<Pos2>> {
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
