use prelude::egui::Response;
use prelude::egui::RichText;
use prelude::egui::Ui;
use prelude::theme;

pub trait TextExt {
    fn themed_text(&mut self, s: impl Into<String>) -> Response;
    fn muted_text(&mut self, s: impl Into<String>) -> Response;
    fn strong_text(&mut self, s: impl Into<String>) -> Response;
}

impl TextExt for Ui {
    fn themed_text(&mut self, s: impl Into<String>) -> Response {
        let t = theme(self.ctx());
        self.label(RichText::new(s.into()).color(t.text()))
    }

    fn muted_text(&mut self, s: impl Into<String>) -> Response {
        let t = theme(self.ctx());
        self.label(RichText::new(s.into()).color(t.text_muted()))
    }

    fn strong_text(&mut self, s: impl Into<String>) -> Response {
        let t = theme(self.ctx());
        self.label(RichText::new(s.into()).color(t.text()).strong())
    }
}
