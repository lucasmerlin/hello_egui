use eframe::egui;
use eframe::egui::{CollapsingHeader, Id, Ui};

use egui_dnd::{dnd, DragDropItem, Handle};

pub fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native("DnD", options, Box::new(|_cc| Ok(Box::<MyApp>::default())))
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
}

impl Default for MyApp {
    fn default() -> Self {
        MyApp {
            items: vec![
                SortableItem {
                    name: "a".to_owned(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "b".to_owned(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "c".to_owned(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "d".to_owned(),
                    ..SortableItem::default()
                },
                SortableItem {
                    name: "e".to_owned(),
                    children: Some(vec![
                        SortableItem {
                            name: "e_a".to_owned(),
                            ..SortableItem::default()
                        },
                        SortableItem {
                            name: "e_b".to_owned(),
                            ..SortableItem::default()
                        },
                        SortableItem {
                            name: "e_c".to_owned(),
                            ..SortableItem::default()
                        },
                        SortableItem {
                            name: "e_d".to_owned(),
                            ..SortableItem::default()
                        },
                    ]),
                },
            ],
        }
    }
}

impl MyApp {
    fn draw_item(ui: &mut Ui, item: &mut SortableItem, handle: Handle<'_>) {
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
        }
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
