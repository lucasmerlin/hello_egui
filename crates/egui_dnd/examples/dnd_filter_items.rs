use std::hash::{Hash, Hasher};
use std::mem;

use eframe::NativeOptions;
use egui::{CentralPanel, ScrollArea, Ui};

use egui_dnd::dnd;
use egui_dnd::utils::shift_vec;

/// There are two ways to filter the list:
///
/// 1. Just skip rendering items
/// You will have to set the item spacing to 0 so the hidden items don't cause additional padding.
#[allow(clippy::ptr_arg)]
fn filter_by_skipping_items(ui: &mut Ui, filter: &str, items: &mut Vec<ItemType>) {
    let spacing = mem::replace(&mut ui.spacing_mut().item_spacing.y, 0.0);

    dnd(ui, "dnd").show_vec(items, |ui, item, handle, _dragging| {
        if !item.number.to_string().contains(filter) {
            return;
        }
        ui.spacing_mut().item_spacing.y = spacing;
        handle.ui(ui, |ui| {
            ui.label(&item.number.to_string());
        });
    });
}

/// 2. Filter the source list
/// This is a bit more complex but will work better if you e.g.
/// use a virtual list to improve performance with a lot of items
#[allow(clippy::ptr_arg)]
fn filter_by_filtering_source_list(ui: &mut Ui, filter: &str, items: &mut Vec<ItemType>) {
    let mut filtered = items
        .iter_mut()
        // We enumerate so we can later get the original index
        .enumerate()
        .filter(|(_, item)| item.number.to_string().contains(filter))
        .collect::<Vec<_>>();

    let response = dnd(ui, "dnd").show(filtered.iter_mut(), |ui, (_, item), handle, _dragging| {
        ui.horizontal(|ui| {
            handle.ui(ui, |ui| {
                ui.label(&item.number.to_string());
            });
        });
    });

    if let Some(update) = response.final_update() {
        // Get the index the item had in the original vec
        let (original_index_from, _) = filtered[update.from];
        let (original_index_to, _) = filtered[update.to];
        // Get the original indices of the items for the update
        shift_vec(original_index_from, original_index_to, items);
    }
}

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

    let mut filter = String::new();

    let mut filter_type = "skip";

    eframe::run_simple_native(
        "dnd filter demo",
        NativeOptions::default(),
        move |ctx, _| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Filter:");
                    ui.text_edit_singleline(&mut filter);

                    ui.label("Filter type:");
                    ui.selectable_value(&mut filter_type, "skip", "skip");
                    ui.selectable_value(&mut filter_type, "filter", "filter");
                });
                ScrollArea::vertical().show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    if filter_type == "skip" {
                        filter_by_skipping_items(ui, &filter, &mut items);
                    } else {
                        filter_by_filtering_source_list(ui, &filter, &mut items);
                    }
                })
            });
        },
    )
}
