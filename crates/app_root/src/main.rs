use hosh_hosh_ui::eframe::App;
use hosh_hosh_ui::eframe::Frame;
use hosh_hosh_ui::eframe::NativeOptions;
use hosh_hosh_ui::eframe::Result;
use hosh_hosh_ui::eframe::run_native;
use hosh_hosh_ui::egui::CentralPanel;
use hosh_hosh_ui::egui::Color32;
use hosh_hosh_ui::egui::Ui;
use hosh_hosh_ui::egui::ViewportBuilder;
use hosh_hosh_ui::*;

fn main() -> Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 620.0])
            .with_title("hosh_hosh_ui — cyberpunk"),
        ..Default::default()
    };

    run_native(
        "hosh_hosh_ui_demo",
        options,
        Box::new(|cc| Ok(Box::new(DemoApp))),
    )
}

struct DemoApp;

impl App for DemoApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                DefaultPrimaryButton {}.render(ui, &CyberpunkTheme, "LAUNCH");
                DefaultPrimaryButton {}.render(ui, &CyberpunkTheme, "HASHIM");
                DefaultPrimaryButton {}.render(ui, &DefaultTheme, "HASHIM");
            });
        });
    }
}

pub struct CyberpunkTheme;

impl Theme for CyberpunkTheme {
    fn primary(&self) -> Color32 {
        Color32::from_rgb(255, 42, 109) // hot magenta
    }
    fn on_primary(&self) -> Color32 {
        Color32::from_rgb(255, 245, 250)
    }
    fn secondary(&self) -> Color32 {
        Color32::from_rgb(5, 217, 232) // cyan
    }
    fn danger(&self) -> Color32 {
        Color32::from_rgb(255, 56, 100)
    }
    fn success(&self) -> Color32 {
        Color32::from_rgb(5, 255, 161)
    }
    fn background(&self) -> Color32 {
        Color32::from_rgb(10, 5, 20)
    }
    fn surface(&self) -> Color32 {
        Color32::from_rgb(20, 9, 30)
    }
    fn surface_alt(&self) -> Color32 {
        Color32::from_rgb(28, 15, 42)
    }
    fn text(&self) -> Color32 {
        Color32::from_rgb(230, 241, 255)
    }
    fn text_muted(&self) -> Color32 {
        Color32::from_rgb(122, 105, 144)
    }
    fn border(&self) -> Color32 {
        Color32::from_rgb(61, 42, 92)
    }
}
