use prelude::egui::Align;
use prelude::egui::Layout;
use prelude::egui::Ui;
use prelude::egui::vec2;

pub fn horizontal(ui: &mut Ui, add_contents: Vec<Box<dyn FnOnce(&mut Ui)>>) {
    let y = ui.available_height();
    let w = ui.available_width();
    let n = add_contents.len();
    if n == 0 {
        return;
    }
    let x = w / n as f32;

    ui.spacing_mut().item_spacing.x = 0.0;

    ui.horizontal(|ui| {
        for component in add_contents {
            ui.allocate_ui_with_layout(vec2(x, y), Layout::top_down(Align::default()), component);
        }
    });
}

pub fn vertical(ui: &mut Ui, add_contents: Vec<Box<dyn FnOnce(&mut Ui)>>) {
    let w = ui.available_width();
    let h = ui.available_height();
    let n = add_contents.len();
    if n == 0 {
        return;
    }
    let y = h / n as f32;

    ui.spacing_mut().item_spacing.y = 0.0;

    ui.vertical(|ui| {
        for component in add_contents {
            ui.allocate_ui_with_layout(vec2(w, y), Layout::top_down(Align::default()), component);
        }
    });
}
