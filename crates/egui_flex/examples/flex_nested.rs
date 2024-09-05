use egui::{CentralPanel, Frame};
use egui_flex::flex_button::FlexButton;
use egui_flex::{Flex, FlexItem};

fn main() -> eframe::Result {
    eframe::run_simple_native("flex nested", Default::default(), |ctx, _frame| {
        CentralPanel::default().show(&ctx, |ui| {
            Flex::horizontal().show(ui, |flex| {
                let frame = Frame::group(flex.ui().style());

                flex.add_frame(FlexItem::new(), frame, |ui| {
                    Flex::vertical().show(ui, |flex| {
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                        flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                    });
                });

                flex.add_container(FlexItem::new(), |ui, content| {
                    content.content(ui, |ui| {
                        Flex::vertical().show(ui, |flex| {
                            flex.add(FlexItem::new(), FlexButton::new("btn"));
                            flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                        });
                    })
                });

                flex.add_frame(FlexItem::new(), frame, |ui| {
                    Flex::vertical().show(ui, |flex| {
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                        flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                    });
                });
            });
        });
    })
}
