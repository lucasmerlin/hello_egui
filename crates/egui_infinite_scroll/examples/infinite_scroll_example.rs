use eframe::{egui, NativeOptions};
use egui::{CentralPanel, ScrollArea};
use egui_infinite_scroll::InfiniteScroll;

pub fn main() -> eframe::Result<()> {
    let mut infinite_scroll = InfiniteScroll::new().end_loader(|cursor, callback| {
        let start = cursor.unwrap_or(0);
        let end = start + 100;
        callback(Ok(((start..end).collect(), Some(end))));
    });

    eframe::run_simple_native(
        "Infinite Scroll Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if ui.button("Reset").clicked() {
                        infinite_scroll.reset();
                    }

                    infinite_scroll.ui(ui, 10, |ui, _index, item| {
                        ui.label(format!("Item {item}"));
                    });
                });
            });
        },
    )
}
