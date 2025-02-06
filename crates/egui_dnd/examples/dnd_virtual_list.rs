use eframe::epaint::Margin;
use eframe::{egui, NativeOptions};
use egui::{CentralPanel, Frame, Id, ScrollArea};
use egui_dnd::dnd;
use egui_virtual_list::VirtualList;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub fn main() -> eframe::Result<()> {
    let mut items: Vec<_> = (0..100_000).collect();
    let mut virtual_list = VirtualList::new();

    eframe::run_simple_native(
        "DnD Virtual List Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    let response = dnd(ui, "dnd").show_custom(|ui, iter| {
                        virtual_list.ui_custom_layout(ui, items.len(), |ui, start_index| {
                            let item = &items[start_index];

                            iter.next(
                                ui,
                                Id::new(*item), // assumes that each item is a unique hash
                                start_index,
                                true,
                                |ui, dnd_item| {
                                    dnd_item.ui(ui, |ui, handle, _item_state| {
                                        draw_item(ui, handle, *item);
                                    })
                                },
                            );

                            1
                        });
                    });

                    // Use update_vec() (or update your data structure yourself on every frame) for
                    // smooth dragging and dropping, as updating based on response.final_update()
                    // after dragging for a long time results in the dragged item failing to render
                    // and the scroll area jumping slightly upon release
                    response.update_vec(&mut items);
                });
            });
        },
    )
}

fn draw_item(ui: &mut egui::Ui, handle: egui_dnd::Handle, item: i32) {
    // For the sake of the example we generate a random height based on the item index
    // but if your row height e.g. depends on some text with varying rows this would also work.
    let mut rng = StdRng::seed_from_u64(item as u64);
    let height = rng.random_range(0..=100);

    Frame::canvas(ui.style())
        .inner_margin(Margin::symmetric(16, (8 + height / 2) as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            handle.ui(ui, |ui| {
                ui.label(format!("Item {item}"));
            });
        });
}
