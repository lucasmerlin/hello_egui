use eframe::NativeOptions;
use egui::{CentralPanel, Id, ScrollArea, Sense};

use egui_dnd::{DragDropItem, DragDropUi};

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
                                if handle
                                    .sense(Sense::click())
                                    .ui(ui, |ui| {
                                        ui.label("grab");
                                        // if ui.button("click me").clicked() {
                                        //     println!("clicked");
                                        // }
                                    }).clicked() {
                                    println!("clicked {}", item.number);
                                }
                                ui.label(&item.number.to_string());
                            });
                        });

                        response.update_vec(&mut items);
                    })
            });


        egui::Window::new("Devzg")
            .show(ctx, |ui| {
                ctx.style_ui(ui);
            });
    })
}
