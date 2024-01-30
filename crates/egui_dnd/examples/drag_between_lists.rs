use egui::{CentralPanel, Id};
use egui_dnd::{dnd, DragDropItem};
use std::hash::{Hash, Hasher};

struct List {
    title: String,
    id: usize,
    cards: Vec<Card>,
}

struct Card {
    id: usize,
    text: String,
}

impl Hash for List {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl Hash for Card {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

fn main() -> eframe::Result<()> {
    let mut id_idx = 0;
    let mut id = || {
        id_idx += 1;
        id_idx
    };
    let mut data = vec![
        List {
            id: id(),
            title: "List 1".to_owned(),
            cards: vec![
                Card {
                    id: id(),
                    text: "Card 1".to_owned(),
                },
                Card {
                    id: id(),
                    text: "Card 2".to_owned(),
                },
                Card {
                    id: id(),
                    text: "Card 3".to_owned(),
                },
            ],
        },
        List {
            id: id(),
            title: "List 2".to_owned(),
            cards: vec![
                Card {
                    id: id(),
                    text: "Card 4".to_owned(),
                },
                Card {
                    id: id(),
                    text: "Card 5".to_owned(),
                },
                Card {
                    id: id(),
                    text: "Card 6".to_owned(),
                },
            ],
        },
    ];

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    dnd(ui, "sort_lists").show_vec(&mut data, |ui, list, handle, state| {
                        ui.vertical(|ui| {
                            handle.ui(ui, |ui| {
                                ui.label(&list.title);
                            });

                            dnd(ui, list.id).show_vec(
                                &mut list.cards,
                                |ui, card, handle, state| {
                                    handle.ui(ui, |ui| {
                                        ui.label(&card.text);
                                    });
                                },
                            );
                        });
                    });
                });

                // dnd(ui, "dnd_example").show_vec(&mut items, |ui, item, handle, state| {
                //     ui.horizontal(|ui| {
                //         handle.ui(ui, |ui| {
                //             if state.dragged {
                //                 ui.label("dragging");
                //             } else {
                //                 ui.label("drag");
                //             }
                //         });
                //         ui.label(*item);
                //     });
                // });
            });
        },
    )
}
