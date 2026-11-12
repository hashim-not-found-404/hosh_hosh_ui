use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::FontId;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Vec2;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// Padding from the shape's bbox.
const PAD: f32 = 8.0;

/// Local xorshift64 — same generator used elsewhere in `style_parts`.
/// Seeded from the caller; **must stay stable across frames** or the
/// ghosts will jitter instead of drifting smoothly.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed | 1)
    }
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 * (1.0 / 16_777_216.0) // [0, 1)
    }
}

/// Smooth 2D offset for one ghost this frame.
///
/// Random per ghost (from `rng`): phase_x, phase_y, freq_x, freq_y,
/// amp_x, amp_y. **Integer frequencies** (1, 2, or 3) make the motion
/// loop cleanly: at `progress = 1` every `sin` is back at `progress = 0`
/// because `sin(x + 2π·f) == sin(x)` for integer `f`. No jump on wrap.
fn smooth_offset(rng: &mut Rng, phase: f32, width: f32) -> Vec2 {
    let phase_x = rng.next_f32() * std::f32::consts::TAU;
    let phase_y = rng.next_f32() * std::f32::consts::TAU;
    let freq_x = 1.0 + (rng.next_f32() * 2.0).floor(); // 1..=3
    let freq_y = 1.0 + (rng.next_f32() * 2.0).floor();
    let amp_x = width * (0.5 + 0.5 * rng.next_f32()); // 0.5..=1.0 × width
    let amp_y = width * (0.5 + 0.5 * rng.next_f32());
    vec2(
        amp_x * (phase * freq_x + phase_x).sin(),
        amp_y * (phase * freq_y + phase_y).sin(),
    )
}

/// Place `text` inside `shape` at the position dictated by `align`,
/// with three chromatic ghosts drifting in random smooth paths around
/// it. `seed` controls which path each ghost takes; keep it stable
/// across frames for smooth motion.
pub fn make(
    shape: &[Pos2],
    text: &str,
    align: Align2,
    color: Color32,
    progress: f32,
    drift_width: f32,
    seed: u64,
) -> Vec<(Pos2, Color32, String)> {
    if shape.len() < 3 {
        return Vec::new();
    }

    // Bounding box.
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

    // Anchor.
    let ax = match align {
        Align2::LEFT_TOP | Align2::LEFT_CENTER | Align2::LEFT_BOTTOM => x_min + PAD,
        Align2::CENTER_TOP | Align2::CENTER_CENTER | Align2::CENTER_BOTTOM => (x_min + x_max) * 0.5,
        _ => x_max - PAD,
    };
    let ay = match align {
        Align2::LEFT_TOP | Align2::CENTER_TOP | Align2::RIGHT_TOP => y_min + PAD,
        Align2::LEFT_CENTER | Align2::CENTER_CENTER | Align2::RIGHT_CENTER => (y_min + y_max) * 0.5,
        _ => y_max - PAD,
    };
    let anchor = pos2(ax, ay);

    // Colors.
    let a = ((color.a() as f32) * 0.75) as u8;
    let r_ghost = Color32::from_rgba_unmultiplied(color.r(), 0, 0, a);
    let g_ghost = Color32::from_rgba_unmultiplied(0, color.g(), 0, a);
    let b_ghost = Color32::from_rgba_unmultiplied(0, 0, color.b(), a);

    // Per-ghost random smooth offset. `progress` clamped here so callers
    // can pass a raw `.fract()` value or an eased 0..1 without worry.
    let phase = progress.clamp(0.0, 1.0) * std::f32::consts::TAU;
    let mut rng = Rng::new(seed);
    let off_r = smooth_offset(&mut rng, phase, drift_width);
    let off_g = smooth_offset(&mut rng, phase, drift_width);
    let off_b = smooth_offset(&mut rng, phase, drift_width);

    let s = text.to_string();
    vec![
        (anchor, color, s.clone()),
        (anchor + off_r, r_ghost, s.clone()),
        (anchor + off_g, g_ghost, s.clone()),
        (anchor + off_b, b_ghost, s),
    ]
}

/// Paint the bundle. Reverse iteration puts the real text on top.
pub fn draw(painter: &Painter, align: Align2, texts: Vec<(Pos2, Color32, String)>, font: FontId) {
    for (pos, color, text) in texts.into_iter().rev() {
        painter.text(pos, align, text, font.clone(), color);
    }
}
