use eframe::{egui, NativeOptions};
use egui::CentralPanel;
use egui_extras::Column;

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
                if ui.button("Reset").clicked() {
                    infinite_scroll.reset();
                }

                ui.set_width(ui.available_width());
                egui_extras::TableBuilder::new(ui)
                    .auto_shrink([false, false])
                    .column(Column::initial(60.0).resizable(true))
                    .column(Column::remainder())
                    .body(|body| {
                        infinite_scroll.ui_table(body, 10, 10.0, |mut ui, item| {
                            ui.col(|ui| {
                                ui.label("Item");
                            });

                            ui.col(|ui| {
                                ui.label(format!("{item}"));
                            });
                        });
                    });
            });
        },
    )
}
