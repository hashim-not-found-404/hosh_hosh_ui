use prelude::egui::{Color32, Painter, Pos2, Rect, Shape, Stroke, pos2};

pub fn draw(painter: &Painter, color: Color32, points: Vec<Pos2>, width: f32) {
    painter.add(Shape::closed_line(points, Stroke::new(width, color)));
}

pub fn make(rect: &Rect, corner: f32) -> Vec<Pos2> {
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
