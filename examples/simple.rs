use eframe::egui::{Context, Id};
use eframe::{egui, App, Frame, NativeOptions};

use egui_dnd::state::{shift_vec, DragDropItem, DragDropUi};

struct DnDApp {
    dnd: DragDropUi,
    items: Vec<ItemType>,
}

impl Default for DnDApp {
    fn default() -> Self {
        DnDApp {
            dnd: DragDropUi::default(),
            items: ["alfred", "bernhard", "christian"]
                .iter()
                .map(|name| ItemType {
                    name: name.to_string(),
                })
                .collect(),
        }
    }
}

struct ItemType {
    name: String,
}

impl DragDropItem for ItemType {
    fn id(&self) -> Id {
        Id::new(&self.name)
    }
}

impl App for DnDApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response =
                self.dnd
                    .ui::<ItemType>(ui, self.items.iter_mut(), |item, ui, handle| {
                        ui.horizontal(|ui| {
                            handle.ui(ui, item, |ui| {
                                ui.label("grab");
                            });

                            ui.label(&item.name);
                        });
                    });

            if let Some(response) = response.completed {
                shift_vec(response.from, response.to, &mut self.items);
            }
        });
    }
}

pub fn main() {
    eframe::run_native(
        "DnD Example",
        NativeOptions::default(),
        Box::new(|_a| Box::new(DnDApp::default())),
    )
}
