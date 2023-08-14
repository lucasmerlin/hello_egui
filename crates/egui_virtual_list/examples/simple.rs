use eframe::egui;
use egui::{CentralPanel, ScrollArea};
use egui_virtual_list::VirtualList;

pub fn main() -> eframe::Result<()> {
    let mut items: Vec<_> = (0..100000).collect();
    let mut virtual_list = VirtualList::new();

    eframe::run_simple_native(
        "Virtual List Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    virtual_list.ui_custom_layout(ui, items.len(), |ui, start_index| {
                        ui.label(format!("Start index: {}", start_index));
                        1
                    });
                });
            });
        },
    )
}
