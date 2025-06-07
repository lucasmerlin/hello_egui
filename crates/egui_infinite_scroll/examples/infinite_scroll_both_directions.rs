use eframe::{egui, NativeOptions};
use egui::{CentralPanel, ScrollArea, Spinner};
use std::thread::{sleep, spawn};

use egui_infinite_scroll::InfiniteScroll;

pub fn main() -> eframe::Result<()> {
    let mut infinite_scroll = InfiniteScroll::new()
        .start_loader(|cursor, callback| {
            let start = cursor.unwrap_or(0);
            let end = start - 100;
            spawn(move || {
                sleep(std::time::Duration::from_secs_f32(0.6));
                callback(Ok(((end..start).collect(), Some(end))));
            });
        })
        .end_loader(|cursor, callback| {
            let start = cursor.unwrap_or(0);
            let end = start + 100;
            spawn(move || {
                sleep(std::time::Duration::from_secs_f32(0.5));
                callback(Ok(((start..end).collect(), Some(end))));
            });
        });

    eframe::run_simple_native(
        "Infinite Scroll Both Directions Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().animated(false).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if ui.button("Reset").clicked() {
                        infinite_scroll.reset();
                    }
                    ui.vertical_centered(|ui| {
                        ui.add_visible(false, Spinner::new());
                    });

                    infinite_scroll.ui(ui, 10, |ui, _index, item| {
                        ui.label(format!("Item {item}"));
                    });

                    ui.vertical_centered(|ui| {
                        ui.add_visible(
                            infinite_scroll.bottom_loading_state().loading(),
                            Spinner::new(),
                        );
                    });
                });
            });
        },
    )
}
