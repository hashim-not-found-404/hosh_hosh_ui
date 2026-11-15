use crate::utils::Node;
use crate::utils::SizedVertical;
use prelude::egui::Align;
use prelude::egui::Layout as EguiLayout;
use prelude::egui::Rect;
use prelude::egui::ScrollArea;
use prelude::egui::Sense;
use prelude::egui::Ui;
use prelude::egui::UiBuilder;
use prelude::egui::pos2;
use prelude::egui::style::ScrollFadeStyle;
use prelude::egui::vec2;

/// Scrollable vertical container.
///
/// Every child implements [`SizedVertical`] — it reports its height in
/// pixels *before* rendering. The layout uses those heights to:
///
/// * Pre-allocate the exact content block in the scroll area, so the
///   scrollbar reflects the true content height on the very first
///   frame (no measurement lag).
/// * Render each child inside a **finite** sub-scope. Without this,
///   children that fill `available_rect_before_wrap()` (like
///   `LayoutVerticalSizedAndClip`) would see the scroll area's
///   infinite height and claim an infinite rect — which makes the
///   scrollbar scroll into blank space below the real children.
/// * Clip each child to both its own sub-rect **and** the scroll
///   viewport, so overflow from a child can never paint into sibling
///   widgets below the scroll area.
///
/// The scroll-edge **fade** (egui draws a soft gradient at the top and
/// bottom of the viewport when content overflows) is disabled here —
/// it looks like a shadow bleeding over the children.
///
/// ```ignore
/// LayoutScroll::new()
///     .child(LayoutVerticalSizedAndClip::default()
///         .child(48.0, fun_label())
///         .child(200.0, fun_list()))
///     .render(ui);
/// ```
#[derive(Default)]
pub struct Layout {
    children: Vec<Box<dyn SizedVertical>>,
    stick_to_bottom: bool,
}

impl Layout {
    pub fn child(mut self, c: impl SizedVertical) -> Self {
        self.children.push(Box::new(c));
        self
    }

    /// If `true`, the scroll area stays pinned to the bottom of the
    /// content — new children appear at the bottom edge, and previous
    /// scrolling is preserved until the user scrolls up. Good for
    /// logs and chat.
    pub fn stick_to_bottom(mut self, stick: bool) -> Self {
        self.stick_to_bottom = stick;
        self
    }
}

impl Node for Layout {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }

        let total_h: f32 = self.children.iter().map(|c| c.high().max(0.0)).sum();

        // Disable egui's scroll-edge fade — the soft gradient it draws
        // at the top and bottom of the viewport reads as a shadow
        // bleeding over the children. Child Uis inherit this style.
        ui.style_mut().spacing.scroll.fade = ScrollFadeStyle {
            strength: 0.0,
            size: 0.0,
        };

        let mut scroll = ScrollArea::vertical().auto_shrink([false, false]);
        if self.stick_to_bottom {
            scroll = scroll.stick_to_bottom(true);
        }

        scroll.show(ui, |ui| {
            let avail = ui.available_rect_before_wrap();
            let width = avail.width();

            // Pre-allocate the exact content block. Gives the scroll
            // area a correct content height on frame 1 and keeps the
            // inner ui's min_rect finite.
            let content_rect = Rect::from_min_size(avail.min, vec2(width, total_h));
            ui.allocate_rect(content_rect, Sense::empty());

            // Render each child inside its own **finite** sub-scope.
            // The finite `max_rect` is what keeps a child — which
            // internally calls `available_rect_before_wrap()` — from
            // seeing the scroll area's infinite height.
            let mut y = avail.top();
            for child in &self.children {
                let h = child.high().max(0.0);
                if h <= 0.0 {
                    continue;
                }
                let child_rect = Rect::from_min_size(pos2(avail.left(), y), vec2(width, h));
                ui.scope_builder(
                    UiBuilder::new()
                        .max_rect(child_rect)
                        .layout(EguiLayout::top_down(Align::default())),
                    |ui| {
                        // Intersect against the *current* clip (which
                        // includes the scroll viewport) before setting
                        // it. `Ui::set_clip_rect` **overwrites** the
                        // clip rect rather than intersecting it, so a
                        // bare `ui.set_clip_rect(child_rect)` would
                        // throw away the scroll viewport — the child
                        // would then paint its full height, spilling
                        // into siblings below the scroll area.
                        let clip = ui.clip_rect().intersect(child_rect);
                        ui.set_clip_rect(clip);
                        child.render(ui);
                    },
                );
                y += h;
            }
        });
    }
}
