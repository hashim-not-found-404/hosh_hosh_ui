use prelude::egui::Align;
use prelude::egui::Layout;
use prelude::egui::Ui;
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
        let n = self.children.len();
        if n == 0 {
            return;
        }
        // Capture before ui.horizontal — inside, available_height is capped.
        let w = ui.available_width();
        let h = ui.available_height();
        let cell_w = w / n as f32;

        ui.horizontal(|ui| {
            // Zero the inter-cell gap so n cells fit exactly w.
            // Setting it inside the closure keeps it scoped to this row.
            ui.spacing_mut().item_spacing.x = 0.0;

            for c in &self.children {
                ui.allocate_ui_with_layout(
                    vec2(cell_w, h),
                    Layout::top_down(Align::default()),
                    |ui| c.render(ui),
                );
            }
        });
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
        let n = self.children.len();
        if n == 0 {
            return;
        }
        let w = ui.available_width();
        let h = ui.available_height();
        let cell_h = h / n as f32;

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            for c in &self.children {
                ui.allocate_ui_with_layout(
                    vec2(w, cell_h),
                    Layout::top_down(Align::default()),
                    |ui| c.render(ui),
                );
            }
        });
    }
}

/// Grid layout — arranges children in rows of equal-width cells.
///
/// `Grid::new(min_width)` decides how many columns fit based on the
/// available width: `cols = floor(available_width / min_width)`.
/// Children are placed row-major; once a row fills up, the next child
/// wraps to a new row. Every cell in a row gets the same width, and
/// every row gets the same height — the same even-division behaviour
/// as `Horizontal` and `Vertical`.
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
}

impl Node for GridFlexAndJustify {
    fn render(&self, ui: &mut Ui) {
        let n = self.children.len();
        if n == 0 {
            return;
        }
        let w = ui.available_width();
        let h = ui.available_height();
        if w <= 0.0 || h <= 0.0 {
            return;
        }

        // cols ≥ 1 so even a viewport narrower than `min_width`
        // still produces one usable column.
        let cols = ((w / self.min_width).floor() as usize).max(1);
        let rows = (n + cols - 1) / cols;
        let cell_w = w / cols as f32;
        let cell_h = h / rows as f32;

        ui.vertical(|ui| {
            // Zero both axes so rows and cells pack tightly — same
            // trick Horizontal / Vertical use, just applied at both
            // nesting levels.
            ui.spacing_mut().item_spacing.y = 0.0;

            for row in 0..rows {
                let row_start = row * cols;
                let row_end = ((row + 1) * cols).min(n);

                // Each row is a fixed-height horizontal strip.
                ui.allocate_ui_with_layout(
                    vec2(w, cell_h),
                    Layout::left_to_right(Align::default()),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for idx in row_start..row_end {
                            ui.allocate_ui_with_layout(
                                vec2(cell_w, cell_h),
                                Layout::top_down(Align::default()),
                                |ui| self.children[idx].render(ui),
                            );
                        }
                    },
                );
            }
        });
    }
}
