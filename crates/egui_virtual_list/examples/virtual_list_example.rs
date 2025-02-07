use eframe::{egui, NativeOptions};
use egui::{CentralPanel, Frame, Margin, ScrollArea};
use egui_virtual_list::VirtualList;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub fn main() -> eframe::Result<()> {
    let items: Vec<_> = (0..100_000).collect();

    // Since the list stores state that is expensive to calculate, we have to store it somewhere in our application.
    let mut virtual_list = VirtualList::new();

    eframe::run_simple_native(
        "Virtual List Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    virtual_list.ui_custom_layout(ui, items.len(), |ui, start_index| {
                        let item = &items[start_index];

                        // For the sake of the example we generate a random height based on the item index
                        // but if your row height e.g. depends on some text with varying rows this would also work.
                        let mut rng = StdRng::seed_from_u64(*item as u64);
                        let height = rng.random_range(0..=100);

                        Frame::canvas(ui.style())
                            .inner_margin(Margin::symmetric(16, 8 + height / 2))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(format!("Item {item}"));
                            });

                        // Return the amount of items that were rendered this row,
                        // so you could vary the amount of items per row
                        1
                    });
                });
            });
        },
    )
}
