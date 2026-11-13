use crate::utils::Node;
use crate::utils::render_into_rects;
use prelude::egui::Rect;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// Row-major grid, columns at least `min_width` wide.
pub fn make_rects(avail: Rect, n: usize, min_width: f32) -> Vec<Rect> {
    if n == 0 || avail.width() <= 0.0 || avail.height() <= 0.0 {
        return Vec::new();
    }

    // Guard against min_width being 0, negative, NaN, or ∞.
    let min_w = if min_width.is_finite() && min_width > 0.0 {
        min_width
    } else {
        1.0
    };

    // `.max(1)` prevents the divide-by-zero hang.
    // `.min(n)` collapses the column count when the grid has fewer
    // items than would fit as full columns — so a single-row grid
    // fills the width instead of leaving dead space on the right.
    let columns = ((avail.width() / min_w).floor() as usize).max(1).min(n);

    let rows = (n + columns - 1) / columns;

    let h_for_element = avail.height() / rows as f32;
    let w_for_element = avail.width() / columns as f32;

    let mut rects = Vec::with_capacity(n);
    for i in 0..n {
        let row = i / columns;
        let col = i % columns;

        let x = avail.left() + col as f32 * w_for_element;
        let y = avail.top() + row as f32 * h_for_element;

        let rect = Rect::from_min_size(pos2(x, y), vec2(w_for_element, h_for_element));
        rects.push(rect);
    }
    rects
}

/// Grid layout — arranges children in rows of equal-width cells.
///
/// `GridFlexAndJustify::new(min_width)` decides how many columns fit
/// based on the available width: `cols = floor(available_width /
/// min_width)`. Children are placed row-major; once a row fills up,
/// the next child wraps to a new row. Every cell in a row gets the
/// same width, every row gets the same height.
pub struct Layout {
    children: Vec<Box<dyn Node>>,
    min_width: f32,
}

impl Layout {
    /// `min_width` is the smallest a single cell can be (px). A 400 px
    /// grid with `min_width = 100.0` fits 4 columns; the same grid at
    /// 300 px fits 3. Values below 1.0 are clamped so the divisor
    /// used at render time is never 0.
    pub fn new(min_width: f32) -> Self {
        Self {
            children: Vec::new(),
            min_width: min_width.max(1.0),
        }
    }

    pub fn child(mut self, c: impl Node) -> Self {
        self.children.push(Box::new(c));
        self
    }

    pub fn child_with_min_width(mut self, min_width: f32, c: impl Node) -> Self {
        self.min_width = self.min_width.max(min_width);
        self.children.push(Box::new(c));
        self
    }
}

impl Node for Layout {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        let rects = make_rects(avail, self.children.len(), self.min_width);
        render_into_rects(ui, &rects, &self.children);
    }
}
