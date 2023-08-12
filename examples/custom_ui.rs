use eframe::egui;
use egui::{CentralPanel, Frame, Stroke};

use egui_dnd::{dnd, DragDropItem};

pub fn main() -> eframe::Result<()> {
    let mut items = vec!["alfred", "bernhard", "christian"];

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.label("Drag and drop the items below");

                dnd(ui, "custom").show_custom_vec(&mut items, |ui, items, iter| {
                    items.iter().enumerate().for_each(|(i, item)| {
                        iter.next(ui, item.id(), i, |ui, item_handle| {
                            let mut frame = Frame::none();

                            if item_handle.state.dragged {
                                frame =
                                    frame.stroke(Stroke::new(1.0, egui::Color32::from_rgb(0, 0, 0)))
                            }

                            frame
                                .show(ui, |ui| {
                                    item_handle.ui(ui, |ui, handle, state| {
                                        handle.ui(ui, |ui| {
                                            if state.dragged {
                                                ui.label("dragging");
                                            } else {
                                                ui.label("drag");
                                            }
                                        });
                                        ui.label(*item);
                                    })
                                })
                                .inner
                        })
                    });
                });

                ui.label("Another label");
            });
        },
    )
}
