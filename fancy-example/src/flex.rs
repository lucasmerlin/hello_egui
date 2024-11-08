use crate::crate_ui::{Crate, CrateUsage};
use crate::demo_area;
use crate::example::{Example, ExampleTrait};
use crate::shared_state::SharedState;
use crate::sidebar::crate_list_ui;
use egui::{Button, Frame, Link, RichText, Ui};
use egui_flex::{Flex, FlexItem};

pub const FLEX_EXAMPLE: Example = Example {
    name: "Flex",
    crates: &[CrateUsage::simple(Crate::EguiFlex)],
    slug: "flex",
    get: || Box::new(FlexExample::new()),
};

pub struct FlexExample {}

impl FlexExample {
    pub fn new() -> Self {
        Self {}
    }
}

impl ExampleTrait for FlexExample {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        demo_area(ui, "Flex", 420.0, |ui| {
            let space = |ui: &mut Ui| {
                ui.add_space(16.0);
                ui.separator();
                ui.add_space(16.0);
            };

            ui.set_width(ui.available_width());

            ui.label("With egui_flex:");
            Flex::horizontal()
                .grow_items(1.0)
                .w_full()
                .wrap(true)
                .show(ui, |flex| {
                    flex.add(FlexItem::new(), Button::new("Short Button"));
                    flex.add(FlexItem::new(), Button::new("Loooooong Button"));
                    flex.add(
                        FlexItem::new(),
                        Button::new(RichText::new("Big Button").heading()),
                    );
                });

            ui.collapsing("Without egui_flex", |ui| {
                ui.horizontal_wrapped(|ui| {
                    let _ = ui.button("Short Button");
                    let _ = ui.button("Loooooong Button");
                    let _ = ui.button(RichText::new("Big Button").heading());
                });
            });

            space(ui);

            ui.label("You can easily add space between items to fill the available space:");
            ui.small("(the same thing would be possible with egui::Sides but the tab order would be messed up)");
            Frame::popup(ui.style()).show(ui, |ui| {
                Flex::new().w_full().show(ui, |flex| {
                    flex.add(
                        FlexItem::new(),
                        Link::new(RichText::new("hello_egui").heading().strong()),
                    );
                    flex.grow();
                    flex.add(FlexItem::new(), Button::new("☰"));
                    flex.add(FlexItem::new(), Button::new("❓"));
                });
            });

            space(ui);

            ui.label("Items are easily justified:");

            crate_list_ui(ui, shared_state);
        });
    }
}
