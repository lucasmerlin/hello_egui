use eframe::egui::{Context, Id};
use eframe::{egui, App, Frame, NativeOptions};

use egui_dnd::{DragDropItem, DragDropUi};

struct DnDApp {
    // DragDropUi stores state about the currently dragged item
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

// We need this to uniquely identify items. You can also implement the Hash trait.
impl DragDropItem for &mut ItemType {
    fn id(&self) -> Id {
        Id::new(&self.name)
    }
}

impl App for DnDApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response =
                // make sure this is called in a vertical layout.
                // Horizontal sorting is not supported yet.
                self.dnd.ui(ui, self.items.iter_mut(), |item, ui, handle, _pressure| {
                    ui.horizontal(|ui| {
                        // Anything in the handle can be used to drag the item
                        let handle_clicked = handle
                            .ui(ui, |ui| {
                            if ui.button("grab").clicked() {
                                println!("clicked {}", item.name);
                            }
                            ui.label("grab");
                        }).clicked();
                        if handle_clicked {
                            println!("handle clicked {}", item.name);
                            println!("I should never be printed");
                        }

                        ui.label(&item.name);
                    });
                });

            response.update_vec(&mut self.items);
        });
    }
}

pub fn main() {
    eframe::run_native(
        "DnD Example",
        NativeOptions::default(),
        Box::new(|_a| Box::<DnDApp>::default()),
    )
    .unwrap();
}
