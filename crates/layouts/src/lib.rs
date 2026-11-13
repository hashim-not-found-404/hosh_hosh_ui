use prelude::egui::Align;
use prelude::egui::Layout;
use prelude::egui::Rect;
use prelude::egui::Sense;
use prelude::egui::Ui;
use prelude::egui::UiBuilder;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// Anything that can paint itself into a `Ui`.
pub trait Node: 'static {
    fn render(&self, ui: &mut Ui);
}

/// Blanket impl: every `Fn(&mut Ui) + 'static` is a leaf node.
/// This is what lets you write `.child(|ui| { ... })` directly.
impl<F> Node for F
where
    F: Fn(&mut Ui) + 'static,
{
    fn render(&self, ui: &mut Ui) {
        (self)(ui);
    }
}

// ─── Low-level: `rows_*` return rects only ─────────────────────────────
//
// These are pure geometry functions. They answer "for `n` items in
// `avail`, what rect does each get?". No `Ui`, no `Painter`, no
// rendering — just Rect math. They're called by the layout structs
// below, *and* directly by `component_list` (or any other consumer).

/// One row per unit of height, stacked top-to-bottom, full width.
pub fn rows_vertical(avail: Rect, n: usize) -> Vec<Rect> {
    if n == 0 {
        return Vec::new();
    }
    let h = avail.height() / n as f32;
    (0..n)
        .map(|i| {
            Rect::from_min_size(
                pos2(avail.left(), avail.top() + i as f32 * h),
                vec2(avail.width(), h),
            )
        })
        .collect()
}

/// One column per unit of width, side-by-side, full height.
pub fn rows_horizontal(avail: Rect, n: usize) -> Vec<Rect> {
    if n == 0 {
        return Vec::new();
    }
    let w = avail.width() / n as f32;
    (0..n)
        .map(|i| {
            Rect::from_min_size(
                pos2(avail.left() + i as f32 * w, avail.top()),
                vec2(w, avail.height()),
            )
        })
        .collect()
}

/// Row-major grid, columns at least `min_width` wide.
pub fn rows_grid(avail: Rect, n: usize, min_width: f32) -> Vec<Rect> {
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

/// Greedy wrap left-to-right. Each child has a minimum width; rows
/// wrap when the next child would overflow. Leftover width in each
/// row is split equally among its children.
pub fn rows_flex(avail: Rect, mins: &[f32]) -> Vec<Rect> {
    if mins.is_empty() || avail.width() <= 0.0 || avail.height() <= 0.0 {
        return Vec::new();
    }
    let w = avail.width();

    // ── Greedy wrap ─────────────────────────────────────────────
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut used = 0.0_f32;
    for (i, &m) in mins.iter().enumerate() {
        let m = m.max(0.0);
        let needed = if current.is_empty() { m } else { used + m };
        if !current.is_empty() && needed > w {
            rows.push(std::mem::take(&mut current));
            current.push(i);
            used = m;
        } else {
            current.push(i);
            used = needed;
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }

    // ── Per-child rects ─────────────────────────────────────────
    let row_h = avail.height() / rows.len() as f32;
    let mut out = vec![Rect::NOTHING; mins.len()];
    for (r, row) in rows.iter().enumerate() {
        let mins_total: f32 = row.iter().map(|&i| mins[i].max(0.0)).sum();
        let gaps = if row.len() > 1 {
            (row.len() - 1) as f32
        } else {
            0.0
        };
        let extra = (w - mins_total - gaps).max(0.0);
        let per = extra / row.len() as f32;

        let mut x = avail.left();
        let y = avail.top() + r as f32 * row_h;
        for &i in row {
            let cell_w = (mins[i].max(0.0) + per).min(w);
            out[i] = Rect::from_min_size(pos2(x, y), vec2(cell_w, row_h));
            x += cell_w;
        }
    }
    out
}

// ─── Layout structs — thin wrappers over `rows_*` ─────────────────────
//
// Each struct:
//   1. computes rects via its `rows_*` helper,
//   2. claims the union of those rects (advances the parent cursor),
//   3. renders each child into its rect via `scope_builder`.

/// Render `children[i]` into `rects[i]`, top-down-centered inside each
/// rect. The union of the rects is claimed on the parent so the cursor
/// advances by the full block.
fn render_into_rects(ui: &mut Ui, rects: &[Rect], children: &[Box<dyn Node>]) {
    if rects.is_empty() || children.is_empty() {
        return;
    }
    let n = rects.len().min(children.len());

    // Claim the block so the parent cursor moves past it.
    let outer = rects[..n]
        .iter()
        .fold(Rect::NOTHING, |acc, r| acc.union(*r));
    ui.allocate_rect(outer, Sense::empty());

    // Render each child inside its rect. `scope_builder` bounds the
    // child's Ui to `max_rect` without advancing the parent cursor
    // (we already advanced it above).
    for (child, rect) in children.iter().zip(rects.iter()) {
        ui.scope_builder(
            UiBuilder::new()
                .max_rect(*rect)
                .layout(Layout::top_down(Align::default())),
            |ui| child.render(ui),
        );
    }
}

#[derive(Default)]
pub struct Horizontal {
    children: Vec<Box<dyn Node>>,
}

impl Horizontal {
    pub fn child(mut self, c: impl Node) -> Self {
        self.children.push(Box::new(c));
        self
    }
}

impl Node for Horizontal {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        let rects = rows_horizontal(avail, self.children.len());
        render_into_rects(ui, &rects, &self.children);
    }
}

#[derive(Default)]
pub struct Vertical {
    children: Vec<Box<dyn Node>>,
}

impl Vertical {
    pub fn child(mut self, c: impl Node) -> Self {
        self.children.push(Box::new(c));
        self
    }
}

impl Node for Vertical {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        let rects = rows_vertical(avail, self.children.len());
        render_into_rects(ui, &rects, &self.children);
    }
}

/// Grid layout — arranges children in rows of equal-width cells.
///
/// `GridFlexAndJustify::new(min_width)` decides how many columns fit
/// based on the available width: `cols = floor(available_width /
/// min_width)`. Children are placed row-major; once a row fills up,
/// the next child wraps to a new row. Every cell in a row gets the
/// same width, every row gets the same height.
pub struct GridFlexAndJustify {
    children: Vec<Box<dyn Node>>,
    min_width: f32,
}

impl GridFlexAndJustify {
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

impl Node for GridFlexAndJustify {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        let rects = rows_grid(avail, self.children.len(), self.min_width);
        render_into_rects(ui, &rects, &self.children);
    }
}

/// Wrap-flex layout. Each child carries a minimum width; the layout
/// places them left-to-right and wraps to the next row when the next
/// child would overflow. Leftover width in each row is shared equally.
///
/// A child whose `min_width` exceeds the available width still
/// renders — clamped to the available width so it doesn't overflow.
///
/// `children` and `mins` are **parallel vectors**: `children[i]` is
/// the node for `mins[i]`. The invariant `children.len() == mins.len()`
/// is upheld by `child()` being the only way to grow either.
#[derive(Default)]
pub struct FlexAndJustify {
    children: Vec<Box<dyn Node>>,
    mins: Vec<f32>,
}

impl FlexAndJustify {
    /// Add a child with a minimum width in pixels. The child grows to
    /// share the row's leftover width, but never shrinks below
    /// `min_width`.
    pub fn child(mut self, min_width: f32, c: impl Node) -> Self {
        self.children.push(Box::new(c));
        self.mins.push(min_width.max(0.0));
        self
    }
}

impl Node for FlexAndJustify {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        let rects = rows_flex(avail, &self.mins);
        render_into_rects(ui, &rects, &self.children);
    }
}
