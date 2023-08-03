use eframe::egui;
use egui::CentralPanel;
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items = vec!["alfred", "bernhard", "christian"];

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                dnd(ui, "dnd_example").show_vec(&mut items, |ui, item, handle, state| {
                    handle.ui(ui, |ui| {
                        if state.dragged {
                            ui.label("dragging");
                        } else {
                            ui.label("drag");
                        }
                    });
                    ui.label(*item);
                });
            });
        },
    )
}
