use prelude::egui::Align;
use prelude::egui::Layout;
use prelude::egui::Rect;
use prelude::egui::Sense;
use prelude::egui::Ui;
use prelude::egui::UiBuilder;

pub trait SizedVertical: Node + 'static {
    fn high(&self) -> f32;
}

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

/// Render `children[i]` into `rects[i]`, top-down-centered inside each
/// rect. The union of the rects is claimed on the parent so the cursor
/// advances by the full block.
pub fn render_into_rects(ui: &mut Ui, rects: &[Rect], children: &[Box<dyn Node>]) {
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
