use egui::{CentralPanel, Frame};
use egui_flex::flex_button::FlexButton;
use egui_flex::{Flex, FlexAlignContent, FlexItem};

fn main() -> eframe::Result {
    eframe::run_simple_native("flex nested", Default::default(), |ctx, _frame| {
        CentralPanel::default().show(&ctx, |ui| {
            Flex::horizontal()
                .align_content(FlexAlignContent::Normal)
                .show(ui, |flex| {
                    let frame = Frame::group(flex.ui().style());

                    flex.add_flex(FlexItem::new(), Flex::vertical(), |flex| {
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                        flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                    });

                    flex.add(FlexItem::new().grow(1.0), FlexButton::new("Single Button"));

                    flex.add_flex(
                        FlexItem::new().grow(1.0),
                        Flex::vertical().align_content(FlexAlignContent::Stretch),
                        |flex| {
                            flex.add(FlexItem::new().grow(1.0), FlexButton::new("btn"));
                            flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                        },
                    );

                    flex.add_flex(FlexItem::new(), Flex::vertical(), |flex| {
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                        flex.add(FlexItem::new(), FlexButton::new("Very long button"));
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                        flex.add(FlexItem::new(), FlexButton::new("btn"));
                    });
                });
        });
    })
}
