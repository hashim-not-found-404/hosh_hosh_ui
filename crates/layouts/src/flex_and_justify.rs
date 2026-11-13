use crate::utils::Node;
use crate::utils::render_into_rects;
use prelude::egui::Rect;
use prelude::egui::Ui;
use prelude::egui::pos2;
use prelude::egui::vec2;

/// Greedy wrap left-to-right. Each child has a minimum width; rows
/// wrap when the next child would overflow. Leftover width in each
/// row is split equally among its children.
pub fn make_rects(avail: Rect, mins: &[f32]) -> Vec<Rect> {
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
pub struct Layout {
    children: Vec<Box<dyn Node>>,
    mins: Vec<f32>,
}

impl Layout {
    /// Add a child with a minimum width in pixels. The child grows to
    /// share the row's leftover width, but never shrinks below
    /// `min_width`.
    pub fn child(mut self, min_width: f32, c: impl Node) -> Self {
        self.children.push(Box::new(c));
        self.mins.push(min_width.max(0.0));
        self
    }
}

impl Node for Layout {
    fn render(&self, ui: &mut Ui) {
        if self.children.is_empty() {
            return;
        }
        let avail = ui.available_rect_before_wrap();
        let rects = make_rects(avail, &self.mins);
        render_into_rects(ui, &rects, &self.children);
    }
}
