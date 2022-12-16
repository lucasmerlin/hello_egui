#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use eframe::egui::{CollapsingHeader, Id, Rect, Ui};

use egui_dnd::{drag_source, drop_target};

fn main() -> () {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(320.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "DnD",
        options,
        Box::new(|_cc| Box::new(MyApp::default())),
    );
}


#[derive(Default)]
struct SortableItem {
    name: String,

    children: Option<Vec<SortableItem>>,

    drag_state: DragState,
}

#[derive(Default)]
struct DragState {
    source_idx: Option<usize>,
    hovering_idx: Option<usize>,
}


struct MyApp {
    items: Vec<SortableItem>,

    drag_state: DragState,
}

impl Default for MyApp {
    fn default() -> Self {
        MyApp {
            drag_state: DragState::default(),
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
                    children: Some(vec![SortableItem {
                        name: "c_a".to_string(),
                        ..SortableItem::default()
                    }, SortableItem {
                        name: "c_b".to_string(),
                        ..SortableItem::default()
                    }]),
                    ..SortableItem::default()
                },
            ],
        }
    }
}

impl MyApp {
    fn draw_item(ui: &mut Ui, item: &mut SortableItem, id: Id) -> Option<Rect> {
        drag_source(ui, id, |ui| {
            ui.label(&item.name);
        }, |ui| {
            if let Some(children) = &mut item.children {
                CollapsingHeader::new("children").default_open(false).show(ui, |ui| {
                    ui.label("Content");

                    MyApp::draw_draggable_items(ui, children, &mut item.drag_state);
                });
            }
        })
    }

    fn draw_draggable_items(ui: &mut Ui, items: &mut Vec<SortableItem>, drag_state: &mut DragState) {
        let mut rects = None;

        let response = drop_target(ui, true, |ui| {
            rects = Some(
                items
                    .iter_mut()
                    .enumerate()
                    .filter_map(|(i, child)| {

                        if Some(i) == drag_state.hovering_idx {
                            ui.separator();
                           // ui.label("meeee!");
                        }

                        let id = Id::new(&child.name);

                        if ui.memory().is_being_dragged(id) {
                            drag_state.source_idx = Some(i);
                        }

                        MyApp::draw_item(ui, child, id)
                            .map(|rect| (i, rect))
                    })
                    .collect::<Vec<_>>()
            );
        }).response;


        if ui.memory().is_anything_being_dragged() {
            let pos = ui.input().pointer.hover_pos();

            if let (Some(pos), Some(rects)) = (pos, rects) {
                let hovering = rects.iter().find(|(i, rect)| {
                    rect.min.y < pos.y && rect.max.y > pos.y
                });


                if let Some((i, rect)) = hovering {
                    let i = if (pos.y > rect.center().y) {
                        i + 1
                    } else {
                        *i
                    };
                    drag_state.hovering_idx = Some(i);
                }
            }
        }

        if let (Some(target_idx), Some(source_idx)) = (drag_state.hovering_idx, drag_state.source_idx) {
            ui.label(format!("hovering: {}", target_idx));


            if ui.input().pointer.any_released() {
                // dbg!();

                // do the drop:

                let target_idx = if source_idx >= target_idx {
                    target_idx
                } else {
                    target_idx - 1
                };

                let item = items.remove(source_idx);
                items.insert(target_idx, item);

                drag_state.source_idx = None;
                drag_state.hovering_idx = None;
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            MyApp::draw_draggable_items(ui, &mut self.items, &mut self.drag_state);
        });
    }
}
