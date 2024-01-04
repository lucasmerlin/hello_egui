use eframe::egui;
use egui::{CentralPanel, Frame, Margin, ScrollArea};
use egui_virtual_list::VirtualList;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

pub fn main() -> eframe::Result<()> {
    let items: Vec<_> = (0..100000).collect();
    let mut virtual_list = VirtualList::new();

    eframe::run_simple_native(
        "Virtual List Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    virtual_list.ui_custom_layout(ui, items.len(), |ui, start_index| {
                        let item = &items[start_index];

                        let mut rng = StdRng::seed_from_u64(*item as u64);
                        // Should be random height based on start_index between 0 and 100
                        let height = rng.gen_range(0.0..=100.0);

                        Frame::canvas(ui.style())
                            .inner_margin(Margin::symmetric(16.0, 8.0 + height / 2.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(format!("Item {}", item));
                            });

                        1
                    });
                });
            });
        },
    )
}
