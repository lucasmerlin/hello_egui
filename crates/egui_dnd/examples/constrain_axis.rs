use eframe::{egui, NativeOptions};
use egui::CentralPanel;
use egui_dnd::{dnd, DragAxis};

pub fn main() -> eframe::Result<()> {
    let mut items = vec!["alfred", "bernhard", "christian", "dominik"];

    eframe::run_ui_native(
        "DnD Constrain Axis Example",
        NativeOptions::default(),
        move |ui, _frame| {
            CentralPanel::default().show(ui, |ui| {
                ui.label("The dragged item can only move vertically:");
                dnd(ui, "dnd_example")
                    .with_drag_axis(DragAxis::Vertical)
                    .show_vec(&mut items, |ui, item, handle, state| {
                        ui.horizontal(|ui| {
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
            });
        },
    )
}
