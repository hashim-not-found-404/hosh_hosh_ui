use hosh_hosh_ui::eframe::App;
use hosh_hosh_ui::eframe::Frame;
use hosh_hosh_ui::eframe::NativeOptions;
use hosh_hosh_ui::eframe::Result;
use hosh_hosh_ui::eframe::run_native;
use hosh_hosh_ui::egui::CentralPanel;
use hosh_hosh_ui::egui::Color32;
use hosh_hosh_ui::egui::Id;
use hosh_hosh_ui::egui::Ui;
use hosh_hosh_ui::egui::ViewportBuilder;
use hosh_hosh_ui::*;

fn main() -> Result {
    let options = NativeOptions {
        viewport: ViewportBuilder::default(),
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
            Vertical::default()
                .child(fun_primary_button())
                .child(fun_secondary_button())
                .child(fun_dropdown())
                .child(fun_toggle())
                .child(fun_input())
                .render(ui);
        });
    }
}

fn fun_secondary_button() -> impl Fn(&mut Ui) {
    |ui| {
        DefaultSecondaryButton::render(ui, &CyberpunkTheme, "SecondaryButton CyberpunkTheme");
    }
}

fn fun_primary_button() -> impl Fn(&mut Ui) {
    |ui| {
        DefaultPrimaryButton::render(ui, &CyberpunkTheme, "PrimaryButton CyberpunkTheme");
    }
}

fn fun_toggle() -> impl Fn(&mut Ui) {
    |ui| {
        let id = Id::unique("demo-toggle");
        let mut on = ui.ctx().data(|d| d.get_temp::<bool>(id).unwrap_or(false));
        if DefaultToggleButton::render(ui, &CyberpunkTheme, &mut on, "POWER").changed() {
            ui.ctx().data_mut(|d| d.insert_temp(id, on));
        }
    }
}

fn fun_dropdown() -> impl Fn(&mut Ui) {
    |ui| {
        let id = Id::unique("demo-dropdown");
        let mut sel = ui.ctx().data(|d| d.get_temp::<usize>(id).unwrap_or(0));
        if DefaultDropdownButton::render(
            ui,
            &CyberpunkTheme,
            &mut sel,
            &["DEBIT", "CREDIT", "TRANSFER"],
            "TYPE: ",
        )
        .changed()
        {
            ui.ctx().data_mut(|d| d.insert_temp(id, sel));
        }
    }
}

fn fun_input() -> impl Fn(&mut Ui) {
    |ui| {
        let id = Id::unique("demo-input");
        let mut text = ui
            .ctx()
            .data(|d| d.get_temp::<String>(id).unwrap_or_default());
        if DefaultInputField::render(ui, &CyberpunkTheme, &mut text, "NAME").changed() {
            ui.ctx().data_mut(|d| d.insert_temp(id, text));
        }
    }
}

pub struct CyberpunkTheme;

impl Theme for CyberpunkTheme {
    // ── Accents ─────────────────────────────────────────────────────
    fn brand(&self) -> Color32 {
        // Steel blue — the one cold accent. Muted, crystalline.
        Color32::from_rgb(110, 175, 210)
    }
    fn on_brand(&self) -> Color32 {
        // Cool charcoal — the secondary's ghost face.
        // Just barely lighter than `background`, so it reads as depth
        // not as a filled shape.
        Color32::from_rgb(22, 26, 34)
    }
    fn brand_alt(&self) -> Color32 {
        // Warm copper — the complementary warm accent.
        // Blue + copper is the classic cinematic pairing.
        Color32::from_rgb(200, 155, 100)
    }
    fn danger(&self) -> Color32 {
        Color32::from_rgb(205, 105, 95)
    }
    fn success(&self) -> Color32 {
        Color32::from_rgb(125, 190, 145)
    }

    // ── Surfaces — cool-tinted, near-black ──────────────────────────
    fn background(&self) -> Color32 {
        Color32::from_rgb(14, 16, 22)
    }
    fn surface(&self) -> Color32 {
        Color32::from_rgb(20, 23, 31)
    }
    fn surface_elevated(&self) -> Color32 {
        Color32::from_rgb(28, 32, 42)
    }

    // ── Foreground ──────────────────────────────────────────────────
    fn text(&self) -> Color32 {
        // Cool off-white — never pure white on dark.
        Color32::from_rgb(220, 226, 234)
    }
    fn text_muted(&self) -> Color32 {
        Color32::from_rgb(130, 138, 152)
    }
    fn border(&self) -> Color32 {
        Color32::from_rgb(52, 58, 72)
    }
}
