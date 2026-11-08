use prelude::egui::Frame;
use prelude::egui::Margin;
use prelude::egui::Stroke;
use prelude::egui::Ui;
use prelude::theme;

pub trait CardExt {
    fn themed_card<R>(&mut self, add: impl FnOnce(&mut Ui) -> R) -> R;
}

impl CardExt for Ui {
    fn themed_card<R>(&mut self, add: impl FnOnce(&mut Ui) -> R) -> R {
        let t = theme(self.ctx());
        Frame::NONE
            .fill(t.surface())
            .stroke(Stroke::new(1.0, t.border()))
            .corner_radius(t.radius())
            .inner_margin(Margin::same(16))
            .show(self, add)
            .inner
    }
}
