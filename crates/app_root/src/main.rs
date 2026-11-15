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
        LayoutVertical::default()
            .child(
                LayoutScroll::default().child(
                    LayoutVerticalSizedAndClip::default()
                        .child(48.0, fun_label())
                        .child(48.0, fun_primary_button())
                        .child(48.0, fun_secondary_button())
                        .child(48.0, fun_toggle())
                        .child(48.0, fun_input())
                        .child(100.0, fun_primary_button())
                        .child(200.0, fun_tertiary_button()),
                ),
            )
            .child(fun_tertiary_button())
            .render(ui);
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
        let on = ui.ctx().data(|d| d.get_temp::<bool>(id).unwrap_or(false));
        if DefaultToggleButton::render(ui, &CyberpunkTheme, on, "POWER").clicked() {
            ui.ctx().data_mut(|d| d.insert_temp(id, on ^ true));
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

fn fun_input_list() -> impl Fn(&mut Ui) {
    |ui| {
        let id = Id::unique("demo-input-list");
        let mut text = ui
            .ctx()
            .data(|d| d.get_temp::<String>(id).unwrap_or_default());
        if DefaultInputList::render(
            ui,
            &CyberpunkTheme,
            &mut text,
            &["DEBIT", "CREDIT", "TRANSFER", "REFUND", "ADJUSTMENT"],
            "TYPE",
        )
        .changed()
        {
            ui.ctx().data_mut(|d| d.insert_temp(id, text));
        }
    }
}

fn fun_label() -> impl Fn(&mut Ui) {
    |ui| {
        DefaultLabel::render(ui, &CyberpunkTheme, "ACCOUNT SETTINGS");
    }
}

fn fun_preview() -> impl Fn(&mut Ui) {
    |ui| {
        render(ui);
    }
}

fn fun_list() -> impl Fn(&mut Ui) {
    |ui| {
        let id = Id::unique("demo-list");
        let mut selected = ui.ctx().data(|d| d.get_temp::<usize>(id).unwrap_or(0));

        let weights = [1.5, 1.5, 1.0];
        let rows: &[[&str; 3]] = &[
            ["Debit", "100.00", "USD"],
            ["Credit", "50.00", "USD"],
            ["Refund", "12.50", "USD"],
        ];

        // The list does not know what layout exists. We tell it.
        let avail = ui.available_rect_before_wrap();
        let rects = make_rects_flex_and_justify(avail, &[50.0, 71.0, 990.0]);

        if DefaultList::render(ui, &CyberpunkTheme, &rects, weights, rows, &mut selected).changed()
        {
            ui.ctx().data_mut(|d| d.insert_temp(id, selected));
            println!("{selected}");
        }
    }
}

fn fun_tertiary_button() -> impl Fn(&mut Ui) {
    |ui| {
        DefaultTertiaryButton::render(ui, &CyberpunkTheme, "TertiaryButton");
    }
}

pub struct CyberpunkTheme;

impl Theme for CyberpunkTheme {
    // ── Accents ─────────────────────────────────────────────────────
    fn brand(&self) -> Color32 {
        Color32::from_rgb(110, 175, 210)
    }
    fn on_brand(&self) -> Color32 {
        Color32::from_rgb(22, 26, 34)
    }
    fn brand_alt(&self) -> Color32 {
        Color32::from_rgb(200, 155, 100)
    }
    fn danger(&self) -> Color32 {
        Color32::from_rgb(205, 105, 95)
    }
    fn success(&self) -> Color32 {
        Color32::from_rgb(125, 190, 145)
    }

    // ── Surfaces ────────────────────────────────────────────────────
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
        Color32::from_rgb(220, 226, 234)
    }
    fn text_muted(&self) -> Color32 {
        Color32::from_rgb(130, 138, 152)
    }
    fn border(&self) -> Color32 {
        Color32::from_rgb(52, 58, 72)
    }
}
