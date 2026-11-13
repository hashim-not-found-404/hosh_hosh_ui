use component_style_parts::draw_border;
use component_style_parts::draw_glitches;
use component_style_parts::draw_glow;
use component_style_parts::draw_text;
use component_style_parts::drow_random_horizontal_lines;
use component_style_parts::fill_shape;
use component_style_parts::is_animation;
use component_style_parts::make_border;
use component_style_parts::make_glitches;
use component_style_parts::make_glow;
use component_style_parts::make_inner_rect;
use component_style_parts::make_random_horizontal_lines;
use component_style_parts::make_text;
use prelude::List;
use prelude::Theme;
use prelude::egui::Align;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::CursorIcon;
use prelude::egui::FontId;
use prelude::egui::Layout;
use prelude::egui::Painter;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Ui;
use prelude::egui::UiBuilder;
use prelude::egui::pos2;
use prelude::egui::vec2;

const CELL_PAD: f32 = 4.0;
const CORNER: f32 = 8.0;
const BORDER_WIDTH: f32 = 1.0;
const HOVER_SECS: f32 = 0.20;
const GLITCH_DENSE: f32 = 400.0;
/// Same drift the primary button uses — keeps the ghost spread
/// consistent across widgets.
const TEXT_DRIFT: f32 = 2.5;
/// `make_text` adds this to the shape's bbox left edge to anchor
/// `Align2::LEFT_CENTER`. Must match `component_style_parts::text::PAD`.
/// We build per-cell shapes offset by `-TEXT_PAD` so the rendered text
/// sits exactly at `x`, i.e. at `CELL_PAD` from the cell edge.
const TEXT_PAD: f32 = 8.0;

pub struct DefaultList;

impl List for DefaultList {
    fn render<const N: usize>(
        ui: &mut Ui,
        theme: &dyn Theme,
        weights: [f32; N],
        labels: &[[&str; N]],
        selected: &mut usize,
    ) -> Response {
        paint(
            ui,
            Color32::WHITE,
            Color32::BLACK,
            weights,
            labels,
            selected,
        )
    }
}

pub fn paint<const N: usize>(
    ui: &mut Ui,
    color_1: Color32,
    color_2: Color32,
    weights: [f32; N],
    labels: &[[&str; N]],
    selected: &mut usize,
) -> Response {
    if labels.is_empty() {
        let (_, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        return response;
    }

    let w = ui.available_width();
    let h = ui.available_height();
    if w <= 0.0 || h <= 0.0 {
        let (_, response) = ui.allocate_exact_size(vec2(w.max(0.0), h.max(0.0)), Sense::hover());
        return response;
    }

    let n = labels.len();
    let cell_h = h / n as f32;

    let (outer, mut response) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
    let mut clicked_idx: Option<usize> = None;

    ui.scope_builder(
        UiBuilder::new()
            .max_rect(outer)
            .layout(Layout::top_down(Align::Min)),
        |ui| {
            for (i, row) in labels.iter().enumerate() {
                let row_resp = ui
                    .allocate_ui_with_layout(vec2(w, cell_h), Layout::top_down(Align::Min), |ui| {
                        paint_row(ui, color_1, color_2, weights, row)
                    })
                    .inner;
                if row_resp.clicked() {
                    clicked_idx = Some(i);
                }
            }
        },
    );

    if let Some(idx) = clicked_idx {
        *selected = idx;
        ui.ctx()
            .data_mut(|d| d.insert_temp(response.id.with("last_clicked"), idx));
        response.mark_changed();
    }

    response
}

fn paint_row<const N: usize>(
    ui: &mut Ui,
    color_1: Color32,
    color_2: Color32,
    weights: [f32; N],
    labels: &[&str; N],
) -> Response {
    let (outer_rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    if outer_rect.width() <= 0.0 || outer_rect.height() <= 0.0 {
        return response;
    }

    let time = ui.input(|i| i.time);
    let seed = time.to_bits();

    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("h"), response.hovered(), HOVER_SECS);
    let is_animate_hover = is_animation(ui, response.id.with("ah"), 0.16, response.hovered());
    let is_animate_click = is_animation(ui, response.id.with("ac"), 0.16, response.clicked());
    let animate = is_animate_hover || is_animate_click;

    let inner_rect = make_inner_rect(&outer_rect);
    let points = make_border(&inner_rect, CORNER);
    let painter = ui.painter();

    // ── Face + border + glow on hover ──────────────────────────
    let text_color = if hover > 0.001 {
        fill_shape(painter, color_1, points.clone());
        draw_border(painter, color_1, points.clone(), BORDER_WIDTH);

        let glows = make_glow(&outer_rect, &points, (hover / 0.25) as u8);
        draw_glow(painter, color_1, &glows);

        color_2
    } else {
        color_1
    };

    // ── Cells — plain text, or animated ghosts during the burst ─
    draw_cells(
        painter,
        inner_rect,
        text_color,
        weights,
        labels,
        time.fract() as f32,
        TEXT_DRIFT,
        seed,
        animate,
    );

    // ── Glitch + random lines during the burst ──────────────────
    if animate {
        let glitches = make_glitches(&points, GLITCH_DENSE, seed);
        let lines = make_random_horizontal_lines(&points, 25.0, seed);
        draw_glitches(painter, text_color, glitches);
        drow_random_horizontal_lines(painter, text_color, lines, 1.0);
    }

    if response.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
    if response.hovered() || animate {
        ui.ctx().request_repaint();
    }

    response
}

fn draw_cells<const N: usize>(
    painter: &Painter,
    rect: Rect,
    color: Color32,
    weights: [f32; N],
    labels: &[&str; N],
    progress: f32,
    drift_width: f32,
    seed: u64,
    animate: bool,
) {
    let mut total: f32 = 0.0;
    for &w in weights.iter() {
        if w.is_finite() && w > 0.0 {
            total += w;
        }
    }
    if total <= 0.0 {
        return;
    }

    let font = FontId::monospace(16.0);
    let cy = rect.center().y;
    let mut x = rect.left() + CELL_PAD;

    for (i, &label) in labels.iter().enumerate() {
        let weight = weights[i];
        if !weight.is_finite() || weight <= 0.0 {
            continue;
        }
        let col_w = rect.width() * weight / total;

        if animate {
            // `make_text` anchors `LEFT_CENTER` at `shape.bbox.left() + PAD`.
            // We want the anchor at `x`, so build a thin quad whose left
            // edge is `x - TEXT_PAD`. The y-extent covers the whole cell
            // so `CENTER` on the y-axis lands at `cy`.
            let left = x - TEXT_PAD;
            let shape = vec![
                pos2(left, rect.top() + 1.0),
                pos2(left + 1.0, rect.top() + 1.0),
                pos2(left + 1.0, rect.bottom() - 1.0),
                pos2(left, rect.bottom() - 1.0),
            ];

            // Distinct seed per cell → each cell's ghosts drift
            // independently, but deterministically per frame.
            let cell_seed = seed ^ ((i as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));

            let bundle = make_text(
                &shape,
                label,
                Align2::LEFT_CENTER,
                color,
                progress,
                drift_width,
                cell_seed,
            );
            draw_text(painter, Align2::LEFT_CENTER, bundle, font.clone());
        } else {
            painter.text(pos2(x, cy), Align2::LEFT_CENTER, label, font.clone(), color);
        }

        x += col_w;
    }
}
