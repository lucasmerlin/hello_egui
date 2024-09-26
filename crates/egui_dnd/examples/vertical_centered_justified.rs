use eframe::{egui, NativeOptions};
use egui::{Button, CentralPanel, Widget};
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items = vec!["alfred", "bernhard", "christian"];

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    dnd(ui, "dnd_example").show_vec(&mut items, |ui, item, handle, state| {
                        handle.ui(ui, |ui| {
                            Button::new(&**item).selected(state.dragged).ui(ui);
                        });
                    });
                });
            });
        },
    )
}
