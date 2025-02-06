use eframe::{egui, NativeOptions};
use egui::{CentralPanel, Frame, Stroke, Ui};

use egui_dnd::{dnd, DragDropItem};

pub fn main() -> eframe::Result<()> {
    let mut items = vec!["alfred", "bernhard", "christian"];

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.label("Drag and drop the items below");

                dnd(ui, "custom").show_custom_vec(&mut items, |ui, items, iter| {
                    items.iter().enumerate().for_each(|(i, item)| {
                        let space_content = |ui: &mut Ui, space| {
                            Frame::NONE
                                .stroke(Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0)))
                                .show(ui, |ui| {
                                    ui.set_min_size(space);
                                });
                        };
                        iter.space_before(ui, item.id(), space_content);

                        iter.next(ui, item.id(), i, false, |ui, item_handle| {
                            item_handle.ui(ui, |ui, handle, state| {
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
                            })
                        });

                        iter.space_after(ui, item.id(), space_content);
                    });
                });

                ui.label("Another label");
            });
        },
    )
}
