use eframe::{egui, NativeOptions};
use egui::CentralPanel;
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items = vec!["alfred", "bernhard", "christian"];

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.label("Drag and drop the items below");

                let response =
                    dnd(ui, "dnd_example").show(items.iter(), |ui, item, handle, state| {
                        handle.ui(ui, |ui| {
                            if state.dragged {
                                ui.label("dragging");
                            } else {
                                ui.label("drag");
                            }
                        });
                        ui.label(*item);
                    });

                if response.is_drag_finished() {
                    response.update_vec(&mut items);
                }

                ui.label("Another label");
            });
        },
    )
}
