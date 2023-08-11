use eframe::egui;
use eframe::egui::{CollapsingHeader, Id, Ui};

use egui_dnd::{dnd, DragDropItem, DragDropUi, Handle};

pub fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native("DnD", options, Box::new(|_cc| Box::<MyApp>::default())).unwrap();
}

#[derive(Default)]
struct SortableItem {
    name: String,

    children: Option<Vec<SortableItem>>,
}

impl DragDropItem for &mut SortableItem {
    fn id(&self) -> Id {
        Id::new(&self.name)
    }
}

struct MyApp {
    items: Vec<SortableItem>,

    drag_drop_ui: DragDropUi,
}

impl Default for MyApp {
    fn default() -> Self {
        MyApp {
            drag_drop_ui: DragDropUi::default(),
            items: vec![
                SortableItem {
                    name: "a".to_string(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "b".to_string(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "c".to_string(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "d".to_string(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "e".to_string(),
                    children: Some(vec![
                        SortableItem {
                            name: "e_a".to_string(),
                            ..SortableItem::default()
                        },
                        SortableItem {
                            name: "e_b".to_string(),
                            ..SortableItem::default()
                        },
                        SortableItem {
                            name: "e_c".to_string(),
                            ..SortableItem::default()
                        },
                        SortableItem {
                            name: "e_d".to_string(),
                            ..SortableItem::default()
                        },
                    ]),
                    ..SortableItem::default()
                },
            ],
        }
    }
}

impl MyApp {
    fn draw_item(ui: &mut Ui, item: &mut SortableItem, handle: Handle) {
        handle.ui(ui, |ui| {
            ui.label(&item.name);
        });

        if let Some(children) = &mut item.children {
            CollapsingHeader::new("children")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label("Content");

                    let response = dnd(ui, &item.name).show(
                        children.iter_mut(),
                        |ui, item, handle, _pressed| {
                            Self::draw_item(ui, item, handle);
                        },
                    );

                    response.update_vec(children);
                });
        };
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response =
                dnd(ui, "dnd_example").show(self.items.iter_mut(), |ui, item, handle, _pressed| {
                    MyApp::draw_item(ui, item, handle);
                });
            response.update_vec(&mut self.items);
        });
    }
}
