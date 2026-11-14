use crate::utils::Node;
use prelude::egui::Align;
use prelude::egui::Layout as EguiLayout;
use prelude::egui::Rect;
use prelude::egui::Sense;
use prelude::egui::Ui;
use prelude::egui::UiBuilder;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// Stack children top-to-bottom with **exact** pixel heights.
///
/// * `height > 0.0` — the child gets exactly that many pixels.
/// * `height <= 0.0` — the child is flexible and shares the leftover
///   space (after fixed-height siblings) equally with other flex
///   children.
///
/// Unlike the previous version, heights are **not clamped** to `avail`.
/// The returned rects may extend past `avail.bottom()`; the caller is
/// expected to clip the output to `avail` when rendering.
pub fn make_rects(avail: Rect, heights: &[f32]) -> Vec<Rect> {
    if heights.is_empty() || avail.width() <= 0.0 || avail.height() <= 0.0 {
        return Vec::new();
    }

    let is_flex = |h: f32| !h.is_finite() || h <= 0.0;

    let fixed_sum: f32 = heights.iter().copied().filter(|h| !is_flex(*h)).sum();
    let flex_count = heights.iter().filter(|h| is_flex(**h)).count();
    let flex_h = if flex_count > 0 {
        ((avail.height() - fixed_sum) / flex_count as f32).max(0.0)
    } else {
        0.0
    };

    let mut rects = Vec::with_capacity(heights.len());
    let mut y = avail.top();

    for &h in heights {
        let child_h = if is_flex(h) { flex_h } else { h };
        rects.push(Rect::from_min_size(
            pos2(avail.left(), y),
            vec2(avail.width(), child_h),
        ));
        y += child_h;
    }

    rects
}

/// Vertical layout with explicit pixel heights per child.
///
/// * Each child renders at **exactly** the height you ask for.
/// * Content that extends past the layout's bottom edge is **clipped**
///   — no shrink, no stretch.
/// * Only the layout's own `avail` block is claimed on the parent, so
///   siblings after it are never pushed by overflow.
///
/// ```ignore
/// LayoutVerticalSizedAndClip::new()
///     .child(64.0, header)       // 64 px — always
///     .child(0.0, content)       // flex — fills leftover
///     .child(40.0, footer)       // 40 px — always (clipped if it
///                                //          doesn't fit)
///     .render(ui);
/// ```
#[derive(Default)]
pub struct Layout {
    children: Vec<Box<dyn Node>>,
    heights: Vec<f32>,
}

impl Layout {
    /// Add a child with a requested height in pixels. Values `<= 0.0`
    /// mean "flex — fill the leftover space".
    pub fn child(mut self, height: f32, c: impl Node) -> Self {
        self.children.push(Box::new(c));
        self.heights.push(height);
        self
    }
}

impl Node for Layout {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        if avail.width() <= 0.0 || avail.height() <= 0.0 {
            return;
        }

        let rects = make_rects(avail, &self.heights);
        if rects.is_empty() {
            return;
        }

        // Claim ONLY `avail` on the parent — overflow never reaches the
        // parent cursor.
        ui.allocate_rect(avail, Sense::empty());

        // Render each child into its full (unclamped) rect, but clip
        // its output to the portion that is visible inside `avail`.
        for (child, rect) in self.children.iter().zip(rects.iter()) {
            let visible = rect.intersect(avail);
            if visible.width() <= 0.0 || visible.height() <= 0.0 {
                // Fully outside — skip, nothing to draw.
                continue;
            }

            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(*rect)
                    .layout(EguiLayout::top_down(Align::default())),
                |ui| {
                    // Intersect the child's painter clip with `visible`.
                    // `set_clip_rect` intersects, so the child sees only
                    // the visible portion of its output.
                    ui.set_clip_rect(visible);
                    child.render(ui);
                },
            );
        }
    }
}
