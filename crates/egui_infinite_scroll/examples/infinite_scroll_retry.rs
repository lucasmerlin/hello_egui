use eframe::egui;
use egui::{CentralPanel, ScrollArea};
use egui_infinite_scroll::{InfiniteScroll, LoadingState};

pub fn main() -> eframe::Result<()> {
    let mut infinite_scroll = InfiniteScroll::new().end_loader(|cursor, callback| {
        let start = cursor.unwrap_or(0);
        let end = start + 100;

        let err = rand::random::<f32>();
        let err = err > 0.7;
        callback(if err {
            Err("Error loading your numbers :( please try again".to_string())
        } else {
            Ok(((start..end).collect(), Some(end)))
        });
    });

    eframe::run_simple_native(
        "Infinite Scroll Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    if ui.button("Reset").clicked() {
                        infinite_scroll.reset();
                    };

                    infinite_scroll.ui(ui, 10, |ui, _index, item| {
                        ui.label(format!("Item {}", item));
                    });

                    match infinite_scroll.bottom_loading_state() {
                        LoadingState::Error(err) => {
                            ui.label("Error:");
                            ui.code(err);
                            if ui.button("Retry").clicked() {
                                infinite_scroll.retry_bottom();
                            };
                        }
                        LoadingState::Loading => {
                            ui.spinner();
                        }
                        _ => {}
                    }
                });
            });
        },
    )
}
