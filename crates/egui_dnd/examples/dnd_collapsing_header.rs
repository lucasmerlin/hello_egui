use eframe::{egui, NativeOptions};
use egui::{CentralPanel, CollapsingHeader, Id, Ui};
use egui_dnd::dnd;

pub fn main() -> eframe::Result<()> {
    let mut items: Vec<String> = vec![
        "alfred".to_string(),
        "bernhard".to_string(),
        "christian".to_string(),
    ];

    eframe::run_simple_native(
        "DnD with CollapsingHeader Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                if ui.button("Add Item").clicked() {
                    //adds an item to test if ui state it preserved
                    items.push(format!("new item {}", items.len()).to_string());
                }

                //the same data can be displayed multiple times
                ui.horizontal(|ui| {
                    for i in 1..=3 {
                        ui.vertical(|ui| {
                            ui.label(format!("List {i}"));
                            list(ui, Id::new("dnd").with(i), &mut items);
                        });
                    }
                });
            });
        },
    )
}

fn list(ui: &mut Ui, dnd_id: Id, items: &mut Vec<String>) {
    dnd(ui, dnd_id)
        //increased animation time to test return animation
        .with_animation_time(1.0)
        .show_vec(items, |ui, item, handle, state| {
            ui.horizontal(|ui| {
                handle.ui(ui, |ui| {
                    if state.dragged {
                        ui.label("dragging");
                    } else {
                        ui.label("drag");
                    }
                });

                CollapsingHeader::new(item.to_string()).show_unindented(ui, |ui| {
                    for i in 1..=5 {
                        ui.label(format!("{i} {item}"));
                    }
                });
            });
        });
}
