pub use crate::border::draw as draw_border;
pub use crate::border::make as make_border;
pub use crate::glitches::draw as draw_glitches;
pub use crate::glitches::make as make_glitches;
pub use crate::glow::draw as draw_glow;
pub use crate::glow::make as make_glow;
pub use crate::grid::draw as draw_grid;
pub use crate::grid::make as make_grid;
pub use crate::random_horizontal_lines::drow as drow_random_horizontal_lines;
pub use crate::random_horizontal_lines::make as make_random_horizontal_lines;
pub use crate::scan_line::draw as draw_scan_line;
pub use crate::scan_line::make as make_scan_line;
use prelude::egui::Color32;
use prelude::egui::Painter;
use prelude::egui::Pos2;
use prelude::egui::Rect;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Shape;
use prelude::egui::Stroke;
use prelude::egui::Ui;

mod border;
mod glitches;
mod glow;
mod grid;
mod random_horizontal_lines;
mod scan_line;

pub fn render(ui: &mut Ui) -> Response {
    let (outer_rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());

    if outer_rect.width() <= 0.0 || outer_rect.height() <= 0.0 {
        return response;
    }

    let painter = ui.painter();
    let time = ui.input(|i| i.time);
    let seed = time.to_bits();

    let inner_rect = make_inner_rect(&outer_rect);
    let points = make_border(&inner_rect, 8.0);
    let glitches = make_glitches(&points, 25.0, seed);
    let lines = make_random_horizontal_lines(&points, 25.0, seed);
    let scan = make_scan_line(&points, time, 48.0);
    let glows = make_glow(&outer_rect, &points, 4);
    let grid = make_grid(&points, 8.0);

    draw_glow(painter, Color32::GREEN, &glows);
    draw_border(painter, Color32::WHITE, points.clone(), 1.0);
    fill_shape(painter, Color32::GREEN, points.clone());
    draw_glitches(painter, Color32::BLACK, glitches);
    drow_random_horizontal_lines(painter, Color32::GRAY, lines, 1.0);
    draw_scan_line(painter, Color32::PURPLE, scan, 1.0);
    draw_grid(painter, Color32::RED, grid, 1.0);

    ui.request_repaint();

    response
}

pub fn fill_shape(painter: &Painter, color: Color32, points: Vec<Pos2>) {
    painter.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

pub fn make_inner_rect(outer_rect: &Rect) -> Rect {
    outer_rect.shrink(8.0)
}
