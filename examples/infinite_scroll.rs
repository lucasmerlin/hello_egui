use eframe::egui;
use egui::{CentralPanel, Id, ScrollArea};
use egui_dnd::dnd;
use egui_infinite_scroll::InfiniteScroll;

pub fn main() -> eframe::Result<()> {
    let mut infinite_scroll = InfiniteScroll::new().end_loader(|cursor, callback| {
        let start = cursor.unwrap_or(0);
        let end = start + 100;
        callback(Ok(((start..end).collect(), Some(end))));
    });

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let response = dnd(ui, "dnd").show_custom(|ui, iter| {
                        infinite_scroll.ui(ui, 10, |ui, index, item| {
                            iter.next(ui, Id::new(*item), index, true, |ui, item_handle| {
                                item_handle.ui(ui, |ui, handle, state| {
                                    ui.horizontal(|ui| {
                                        handle.ui(ui, |ui| {
                                            if state.dragged {
                                                ui.label("dragging");
                                            } else {
                                                ui.label("drag");
                                            }
                                        });
                                        ui.label(format!("Item {}", item));
                                    });
                                })
                            });
                        });
                    });

                    response.update_vec(&mut infinite_scroll.items);
                });
            });
        },
    )
}
