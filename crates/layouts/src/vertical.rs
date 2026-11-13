use crate::utils::Node;
use crate::utils::render_into_rects;
use prelude::egui::Rect;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// One row per unit of height, stacked top-to-bottom, full width.
pub fn make_rects(avail: Rect, n: usize) -> Vec<Rect> {
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

#[derive(Default)]
pub struct Layout {
    children: Vec<Box<dyn Node>>,
}

impl Layout {
    pub fn child(mut self, c: impl Node) -> Self {
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
        let rects = make_rects(avail, self.children.len());
        render_into_rects(ui, &rects, &self.children);
    }
}
