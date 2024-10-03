use eframe::NativeOptions;
use egui::{Button, CentralPanel};
use egui_flex::{item, Flex, FlexAlignContent};

fn main() -> eframe::Result {
    eframe::run_simple_native(file!(), NativeOptions::default(), |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            Flex::horizontal().show(ui, |flex| {
                flex.add(item().grow(1.0), Button::new("Growing button"));
                flex.add(item(), Button::new("Non-growing button"));

                // Nested flex
                flex.add_flex(
                    item().grow(1.0),
                    // We need the FlexAlignContent::Stretch to make the buttons fill the space
                    Flex::vertical().align_content(FlexAlignContent::Stretch),
                    |flex| {
                        flex.add(item(), Button::new("Vertical button"));
                        flex.add(item(), Button::new("Another Vertical button"));
                    },
                );
            });
        });
    })
}
