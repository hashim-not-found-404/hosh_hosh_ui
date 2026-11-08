use hosh_hosh_ui::eframe::App;
use hosh_hosh_ui::eframe::Frame;
use hosh_hosh_ui::eframe::NativeOptions;
use hosh_hosh_ui::eframe::Result;
use hosh_hosh_ui::eframe::run_native;
use hosh_hosh_ui::egui::CentralPanel;
use hosh_hosh_ui::egui::Ui;
use hosh_hosh_ui::egui::ViewportBuilder;
use hosh_hosh_ui::*;

fn main() -> Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 620.0])
            .with_title("hosh_hosh_ui — demo"),
        ..Default::default()
    };

    run_native(
        "hosh_hosh_ui_demo",
        options,
        Box::new(|cc| {
            // ── ONE LINE to theme the whole app ────────────────
            set_theme(&cc.egui_ctx, FallbackTheme);
            Ok(Box::new(DemoApp))
        }),
    )
}

struct DemoApp;

impl App for DemoApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        CentralPanel::default().show(ui, |ui| {
            ui.add_space(16.0);
            ui.heading("hosh_hosh_ui");
            ui.muted_text("A tiny themeable UI toolkit for egui.");
            ui.add_space(20.0);

            ui.themed_card(|ui| {
                ui.strong_text("Buttons");
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.primary_button("Save");
                    ui.secondary_button("Cancel");
                    ui.ghost_button("More…");
                    ui.danger_button("Delete");
                });
            });

            ui.add_space(20.0);

            ui.themed_card(|ui| {
                ui.strong_text("Text");
                ui.add_space(10.0);
                ui.themed_text("This is the themed body color.");
                ui.muted_text("This is the muted color, for hints and metadata.");
            });
        });
    }
}
