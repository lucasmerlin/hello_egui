use eframe::NativeOptions;
use egui::{CentralPanel, ScrollArea, Sense};
use std::hash::{Hash, Hasher};

use egui_dnd::dnd;

struct ItemType {
    number: u32,
}

impl Hash for ItemType {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.number.hash(state);
    }
}

fn main() -> eframe::Result<()> {
    let mut items: Vec<_> = (0..1000).map(|number| ItemType { number }).collect();

    eframe::run_simple_native(
        "dnd scroll demo",
        NativeOptions::default(),
        move |ctx, _| {
            CentralPanel::default().show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    dnd(ui, "dnd").show_vec(&mut items, |ui, item, handle, _dragging| {
                        ui.horizontal(|ui| {
                            let clicked = handle
                                .sense(Sense::click())
                                .ui(ui, |ui| {
                                    ui.label("grab");
                                })
                                .clicked();
                            if clicked {
                                println!("clicked {}", item.number);
                            }
                            ui.label(&item.number.to_string());
                        });
                    });
                })
            });
        },
    )
}
