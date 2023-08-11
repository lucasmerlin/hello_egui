use eframe::egui;
use egui::CentralPanel;
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items: Vec<_> = (1..100).collect();

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    items.iter_mut().for_each(|item| {
                        ui.scope(|ui| {
                            ui.label(item.to_string());
                        });
                    });
                });

                ui.horizontal_wrapped(|ui| {
                    dnd(ui, "dnd_example").show_vec(&mut items, |ui, item, handle, state| {
                        handle.ui(ui, |ui| {
                            ui.label("drag");
                        });
                        ui.label(item.to_string());
                    });
                });

                ctx.style_ui(ui);
            });
        },
    )
}
