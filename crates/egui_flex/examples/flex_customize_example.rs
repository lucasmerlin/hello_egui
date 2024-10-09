use eframe::NativeOptions;
use egui::{Button, Window};
use egui_flex::{Flex, FlexInstance, FlexItem};

#[derive(Clone, Debug)]
enum ItemKind {
    Button(String),
    Label(String),
}

impl Default for ItemKind {
    fn default() -> Self {
        Self::Label("Hello World!".to_owned())
    }
}

#[derive(Clone, Debug, Default)]
struct Item {
    kind: ItemKind,
    flex: FlexItem,
}

impl Item {
    pub fn show(&mut self, ui: &mut FlexInstance) {
        let response = match &self.kind {
            ItemKind::Button(text) => ui.add(self.flex, Button::new(text)).inner,
            ItemKind::Label(text) => ui.add_ui(self.flex, |ui| ui.label(text)).inner,
        };

        response.context_menu(|ui| {
            ui.set_width(200.0);
            ui.set_height(100.0);
            Flex::vertical().show(ui, |flex| {
                flex.add_ui(FlexItem::new(), |ui| {
                    Flex::horizontal().show(ui, |flex| {
                        let response = flex
                            .add(
                                FlexItem::new().grow(1.0),
                                Button::new("Label")
                                    .selected(matches!(self.kind, ItemKind::Label(_))),
                            )
                            .inner;

                        if response.clicked() {
                            self.kind = ItemKind::Label("Hello World!".to_owned());
                        }

                        let response = flex
                            .add(
                                FlexItem::new().grow(1.0),
                                Button::new("Button")
                                    .selected(matches!(self.kind, ItemKind::Button(_))),
                            )
                            .inner;

                        if response.clicked() {
                            self.kind = ItemKind::Button("Hello World!".to_owned());
                        }
                    });
                });
                flex.add_ui(FlexItem::new(), |ui| match &mut self.kind {
                    ItemKind::Button(text) => {
                        ui.label("Button Text:");
                        ui.text_edit_multiline(text);
                    }
                    ItemKind::Label(text) => {
                        ui.label("Label Text:");
                        ui.text_edit_multiline(text);
                    }
                });
            });
        });
    }
}

#[derive(Clone, Debug)]
struct Demo {
    items: Vec<Item>,
    flex: Flex,
}

impl Default for Demo {
    fn default() -> Self {
        Self {
            items: vec![
                Item {
                    kind: ItemKind::Button("Button 1".to_owned()),
                    flex: FlexItem::default().grow(1.0),
                },
                Item {
                    kind: ItemKind::Button("Button 2".to_owned()),
                    flex: FlexItem::default().grow(1.0),
                },
                Item {
                    kind: ItemKind::Label("Label 1".to_owned()),
                    flex: FlexItem::default().grow(1.0),
                },
                Item {
                    kind: ItemKind::Label("Label 2".to_owned()),
                    flex: FlexItem::default().grow(1.0),
                },
            ],
            flex: Flex::default(),
        }
    }
}

fn main() -> eframe::Result<()> {
    let mut demo = Demo::default();
    eframe::run_simple_native(
        "flex custom",
        NativeOptions::default(),
        move |ctx, _frame| {
            Window::new("Flex Content").show(ctx, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());
                demo.flex.clone().show(ui, |flex| {
                    for item in &mut demo.items {
                        item.show(flex);
                    }
                });
            });
        },
    )
}
