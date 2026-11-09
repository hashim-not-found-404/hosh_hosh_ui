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
