use hosh_hosh_ui::eframe::App;
use hosh_hosh_ui::eframe::Frame;
use hosh_hosh_ui::eframe::NativeOptions;
use hosh_hosh_ui::eframe::Result;
use hosh_hosh_ui::eframe::run_native;
use hosh_hosh_ui::egui::Align;
use hosh_hosh_ui::egui::CentralPanel;
use hosh_hosh_ui::egui::Color32;
use hosh_hosh_ui::egui::Layout;
use hosh_hosh_ui::egui::Ui;
use hosh_hosh_ui::egui::ViewportBuilder;
use hosh_hosh_ui::egui::vec2;
use hosh_hosh_ui::*;

fn main() -> Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 620.0])
            .with_min_inner_size([300.0, 400.0])
            .with_title("hosh_hosh_ui — responsive"),
        ..Default::default()
    };

    run_native(
        "hosh_hosh_ui_demo",
        options,
        Box::new(|_| Ok(Box::new(DemoApp))),
    )
}

struct DemoApp;

impl App for DemoApp {
    fn ui(&mut self, ui: &mut Ui, _: &mut Frame) {
        CentralPanel::default().show(ui, |ui| {
            horizontal(
                ui,
                vec![
                    Box::new(|ui| {
                        horizontal(
                            ui,
                            vec![Box::new(|ui| {
                                DefaultPrimaryButton::render(ui, &CyberpunkTheme, "FIXED 1");
                            })],
                        );
                    }),
                    Box::new(|ui| {
                        horizontal(
                            ui,
                            vec![
                                Box::new(|ui| {
                                    vertical(
                                        ui,
                                        vec![
                                            Box::new(|ui| {
                                                if DefaultPrimaryButton::render(
                                                    ui,
                                                    &CyberpunkTheme,
                                                    "FIXED 2",
                                                )
                                                .clicked()
                                                {
                                                    println!("hiiiiiiiiiii")
                                                };
                                            }),
                                            Box::new(|ui| {
                                                if DefaultPrimaryButton::render(
                                                    ui,
                                                    &CyberpunkTheme,
                                                    "FIXED 3",
                                                )
                                                .clicked()
                                                {
                                                    println!("hiiiiiiiiiii")
                                                };
                                            }),
                                        ],
                                    );
                                }),
                                Box::new(|ui| {
                                    DefaultPrimaryButton::render(ui, &CyberpunkTheme, "FIXED 4")
                                        .clicked();
                                }),
                            ],
                        );
                    }),
                ],
            );
        });
    }
}

pub struct CyberpunkTheme;

impl Theme for CyberpunkTheme {
    fn primary(&self) -> Color32 {
        Color32::from_rgb(255, 42, 109)
    }
    fn on_primary(&self) -> Color32 {
        Color32::from_rgb(255, 245, 250)
    }
    fn secondary(&self) -> Color32 {
        Color32::from_rgb(5, 217, 232)
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
