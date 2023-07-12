use eframe::NativeOptions;
use egui::{CentralPanel, Id, ScrollArea, Sense};

use egui_dnd::{DragDropItem, DragDropUi};
use egui_dnd::utils::shift_vec;

struct ItemType {
    number: u32,
}

impl DragDropItem for ItemType {
    fn id(&self) -> Id {
        Id::new(&self.number)
    }
}

fn main() -> eframe::Result<()> {
    let mut dnd = DragDropUi::default();

    let mut items: Vec<_> = (0..1000)
        .map(|number| ItemType {
            number,
        })
        .collect();

    eframe::run_simple_native("dnd scroll demo", NativeOptions::default(), move |ctx, _| {
        CentralPanel::default()
            .show(ctx, |ui| {
                ScrollArea
                ::vertical()
                    .show(ui, |ui| {
                        let response = dnd.ui::<ItemType, _>(ui, items.iter_mut(), |item, ui, handle, dragging| {
                            ui.horizontal(|ui| {
                                if handle.ui_impl(ui, Some(Sense::click()), |ui| {
                                    ui.label("grab");
                                }).clicked() {
                                    println!("clicked {}", item.number);
                                }
                                ui.label(&item.number.to_string());
                            });
                        });

                        if let Some(response) = response.completed {
                            let from = response.from;
                            let to = response.to;
                            shift_vec(from, to, &mut items)
                        }
                    })
            });
    })
}
