use prelude::Label;
use prelude::Theme;
use prelude::egui::Align2;
use prelude::egui::Color32;
use prelude::egui::FontId;
use prelude::egui::Response;
use prelude::egui::Sense;
use prelude::egui::Ui;

pub struct DefaultLabel;

impl Label for DefaultLabel {
    fn render(ui: &mut Ui, t: &dyn Theme, text: &str) -> Response {
        paint(ui, text, Color32::WHITE)
    }
}

pub fn paint(ui: &mut Ui, text: &str, color_1: Color32) -> Response {
    let (outer, response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    if outer.width() <= 0.0 || outer.height() <= 0.0 || text.is_empty() {
        return response;
    }

    let painter = ui.painter();

    painter.text(
        outer.left_center(),
        Align2::LEFT_CENTER,
        text,
        FontId::monospace(16.0),
        color_1,
    );

    response
}
