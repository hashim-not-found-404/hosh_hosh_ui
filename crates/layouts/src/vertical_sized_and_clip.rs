use crate::utils::Node;
use crate::utils::SizedVertical;
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
/// Heights are **not clamped** to `avail`. The returned rects may
/// extend past `avail.bottom()`; the caller is expected to clip the
/// output to `avail` when rendering.
///
/// Inside a `ScrollArea`, `avail.height()` is infinite — flex children
/// collapse to height `0` in that case rather than trying to fill
/// infinity.
pub fn make_rects(avail: Rect, heights: &[f32]) -> Vec<Rect> {
    if heights.is_empty() || avail.width() <= 0.0 || avail.height() <= 0.0 {
        return Vec::new();
    }

    let is_flex = |h: f32| !h.is_finite() || h <= 0.0;

    let fixed_sum: f32 = heights.iter().copied().filter(|h| !is_flex(*h)).sum();
    let flex_count = heights.iter().filter(|h| is_flex(**h)).count();

    // Inside a ScrollArea, `avail.height()` is infinite — flexing to
    // fill an infinite space is meaningless, so fall back to 0.
    let flex_h = if flex_count > 0 && avail.height().is_finite() {
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
/// * The layout claims only its **actual content height** on the
///   parent — inside a `ScrollArea` this is essential (claiming
///   `avail` there would make the scroll content infinitely tall).
/// * Each child's clip is **intersected** with the layout's own
///   inherited clip — so when the layout runs inside a scroll area
///   whose viewport is smaller than the content, children that
///   overflow the viewport are cut there, not at the layout's own
///   bottom edge.
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
    pub fn new() -> Self {
        Self::default()
    }

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

        // Claim only the *actual* content height. Inside a ScrollArea
        // `avail.height()` is infinite — claiming the full avail would
        // make the scroll content infinitely tall, so scrolling past
        // the real children shows blank.
        let content_h = rects
            .last()
            .map(|r| r.bottom() - avail.top())
            .unwrap_or(0.0);
        let claim_h = if avail.height().is_finite() {
            content_h.min(avail.height())
        } else {
            content_h
        };
        let claim_rect = Rect::from_min_size(avail.min, vec2(avail.width(), claim_h));
        ui.allocate_rect(claim_rect, Sense::empty());

        // Remember the clip we inherited from our parent. When this
        // layout lives inside a ScrollArea, that clip is the scroll
        // viewport — and it must survive all the way to the leaf
        // painters, otherwise a child that overflows the viewport
        // bleeds into siblings below the scroll area.
        let inherited_clip = ui.clip_rect();

        for (child, rect) in self.children.iter().zip(rects.iter()) {
            let visible = rect.intersect(avail);
            if visible.width() <= 0.0 || visible.height() <= 0.0 {
                continue;
            }

            // `Ui::set_clip_rect` **overwrites** — it does not
            // intersect. If we just call `set_clip_rect(visible)`,
            // we replace the parent's clip (e.g. the scroll viewport)
            // with the layout's own bottom edge, and overflowing
            // children paint past the viewport. Intersect instead so
            // the leaf painter sees `viewport ∩ child_rect`.
            let clip = inherited_clip.intersect(visible);

            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(*rect)
                    .layout(EguiLayout::top_down(Align::default())),
                |ui| {
                    ui.set_clip_rect(clip);
                    child.render(ui);
                },
            );
        }
    }
}

impl SizedVertical for Layout {
    fn high(&self) -> f32 {
        self.heights.iter().sum()
    }
}
