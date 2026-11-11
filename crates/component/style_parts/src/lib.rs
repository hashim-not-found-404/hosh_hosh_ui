use prelude::egui::{Color32, Painter, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, pos2};
mod glitches;

use glitches::make_glitch_pixels;

pub fn render(ui: &mut Ui) -> Response {
    let (outer_rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

    if outer_rect.width() <= 0.0 || outer_rect.height() <= 0.0 {
        return response;
    }

    let color = Color32::WHITE;
    let width = 1.0;
    let corner = 8.0;

    let painter = ui.painter();

    let inner_rect = make_inner_rect(outer_rect);
    let points = make_border_shape(inner_rect, corner);
    let glitches = make_glitch_pixels(points.clone(), 25.0, 1000);

    draw_border(&painter, points.clone(), color, width);
    fill_shape(&painter, points.clone(), Color32::GREEN);
    draw_glitches(painter, glitches, Color32::BLACK);

    response
}

fn draw_border(painter: &Painter, points: Vec<Pos2>, color: Color32, width: f32) {
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
}

fn fill_shape(painter: &Painter, points: Vec<Pos2>, color: Color32) {
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

fn draw_glitches(painter: &Painter, points: Vec<Pos2>, color: Color32) {
    for p in points {
        painter.rect_filled(
            Rect::from_min_max(p, pos2(p.x + 1.0, p.y + 1.0)),
            0.0,
            color,
        );
    }
}

fn make_border_shape(rect: Rect, corner: f32) -> Vec<Pos2> {
    let corner = corner
        .min(rect.width() / 2.0)
        .min(rect.height() / 2.0)
        .max(0.0);

    vec![
        pos2(rect.left(), rect.top()),
        pos2(rect.right() - corner, rect.top()),
        pos2(rect.right(), rect.top() + corner),
        pos2(rect.right(), rect.bottom()),
        pos2(rect.right() - corner, rect.bottom()),
        pos2(rect.left() + corner, rect.bottom()),
        pos2(rect.left(), rect.bottom() - corner),
        pos2(rect.left(), rect.top() + corner),
    ]
}

fn make_inner_rect(outer_rect: Rect) -> Rect {
    outer_rect.shrink(8.0)
}

fn make_random_horizontal_lines(shape: Vec<Pos2>, density: f32, seed: u64) -> Vec<(Pos2, Pos2)> {
    todo!()
}

fn drow_lines(painter: &Painter, lines: Vec<(Pos2, Pos2)>, color: Color32, width: f32) {
    todo!()
}
