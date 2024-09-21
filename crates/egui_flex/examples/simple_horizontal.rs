use egui::{Button, CentralPanel};
use egui_flex::{Flex, FlexItem};

fn main() -> eframe::Result {
    eframe::run_simple_native("simple_horizontal", Default::default(), |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            Flex::horizontal().show(ui, |flex| {
                for i in 0..10 {
                    flex.add(FlexItem::new(), Button::new(format!("Button {}", i)));
                }

                flex.add(FlexItem::new(), Button::new("Button \n 2 line"));
            });
        });
    })
}
