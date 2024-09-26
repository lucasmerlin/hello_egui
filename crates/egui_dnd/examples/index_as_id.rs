use eframe::{egui, NativeOptions};
use egui::{CentralPanel, Id};
use egui_dnd::{dnd, DragDropItem};

#[derive(Clone, Hash)]
struct Item {
    name: String,
}

impl Item {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

// In order to use the index as id we need to implement DragDropItem for a wrapper struct
struct EnumeratedItem<T> {
    item: T,
    index: usize,
}

impl<T> DragDropItem for EnumeratedItem<T> {
    fn id(&self) -> Id {
        Id::new(self.index)
    }
}

pub fn main() -> eframe::Result<()> {
    let mut items = vec![
        Item::new("alfred"),
        Item::new("bernhard"),
        Item::new("christian"),
        Item::new("alfred"),
    ];

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                let response = dnd(ui, "dnd_example")
                    // Since egui_dnd's animations rely on the ids not
                    // changing after the drag finished we need to disable animations
                    .with_animation_time(0.0)
                    .show(
                        items
                            .iter_mut()
                            .enumerate()
                            .map(|(i, item)| EnumeratedItem { item, index: i }),
                        |ui, item, handle, state| {
                            ui.horizontal(|ui| {
                                handle.ui(ui, |ui| {
                                    if state.dragged {
                                        ui.label("dragging");
                                    } else {
                                        ui.label("drag");
                                    }
                                });
                                ui.label(&item.item.name);
                            });
                        },
                    );

                // Since the item id may not change while a drag is ongoing we need to wait
                // until the drag is finished before updating the items
                if response.is_drag_finished() {
                    response.update_vec(&mut items);
                }
            });
        },
    )
}
