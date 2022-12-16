use eframe::{App, egui, Frame, NativeOptions};
use eframe::egui::{Context, Id};

use egui_dnd::state::{DragDropItem, DragDropUi, shift_vec};

struct DnDApp {
    dnd: DragDropUi,
    items: Vec<ItemType>,
}

impl Default for DnDApp {
    fn default() -> Self {
        DnDApp {
            dnd: DragDropUi::default(),
            items: ["alfred", "bernhard", "christian"].iter().map(|name| ItemType {
                name: name.to_string()
            }).collect(),
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
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = self.dnd.ui::<ItemType>(ui, self.items.iter_mut(), |item, ui| {
                ui.label(&item.name);
            });

            if let Some(response) = response.completed {
                shift_vec(response.from, response.to, &mut self.items);
            }
        });
    }
}


pub fn main() {
    eframe::run_native("DnD Example", NativeOptions::default(), Box::new(|a| {
        Box::new(DnDApp::default())
    }))
}
