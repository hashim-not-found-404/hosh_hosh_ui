use crate::utils::Node;
use prelude::egui;
use prelude::egui::Align;
use prelude::egui::Rect;
use prelude::egui::Sense;
use prelude::egui::Ui;
use prelude::egui::UiBuilder;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// Compute the centered rect of size `width × high` inside `avail`.
///
/// A non-positive `width` or `high` means "use the full available
/// dimension" — so `make_rects(avail, 0.0, 0.0)` returns `avail`
/// unchanged (max width, max height). A `width` / `high` larger than
/// `avail` allows is clamped to fit.
pub fn make_rects(avail: Rect, width: f32, high: f32) -> Rect {
    let w = if width > 0.0 {
        width.min(avail.width())
    } else {
        avail.width()
    };
    let h = if high > 0.0 {
        high.min(avail.height())
    } else {
        avail.height()
    };
    let x = avail.left() + (avail.width() - w) * 0.5;
    let y = avail.top() + (avail.height() - h) * 0.5;
    Rect::from_min_size(pos2(x, y), vec2(w, h))
}

/// Center layout — renders a single child inside a centered rect of
/// size `width × high`.
///
/// * `Layout::new(0.0, 0.0, child)` claims the **maximum width and
///   maximum height** of the parent (the rect is the parent's rect).
/// * `Layout::new(320.0, 200.0, child)` produces a `320 × 200` box
///   centered both horizontally and vertically inside the parent.
///
/// The parent cursor advances past the centered rect, so this layout
/// behaves like any other `Node`.
pub struct Layout {
    width: f32,
    high: f32,
    children: Box<dyn Node>,
}

impl Layout {
    /// `width` and `high` are the desired size of the centered box in
    /// pixels. A value `<= 0.0` means "take the full available
    /// dimension" (max width / max height).
    pub fn new(width: f32, high: f32, c: impl Node) -> Self {
        Self {
            width: width.max(0.0),
            high: high.max(0.0),
            children: Box::new(c),
        }
    }
}

impl Node for Layout {
    fn render(&self, ui: &mut Ui) {
        let avail = ui.available_rect_before_wrap();
        let rect = make_rects(avail, self.width, self.high);

        // Claim the centered rect on the parent so the cursor advances
        // past the whole block.
        ui.allocate_rect(rect, Sense::empty());

        // Render the child inside the centered rect.
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(Align::default())),
            |ui| self.children.render(ui),
        );
    }
}
