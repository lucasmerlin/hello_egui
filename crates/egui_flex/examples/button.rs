use egui::emath::TSTransform;
use egui::{Frame, Label, Response, Sense};
use egui_flex::{Flex, FlexInstance, FlexItem, FlexWidget};
use hello_egui_utils::run;

fn main() {
    run!(move |ui| {
        Flex::horizontal().w_full().show(ui, |flex| {
            flex.add(FlexItem::new().grow(1.0), Button::new("Hi"));
            flex.add(FlexItem::new().grow(1.0), Button::new("Hi"));
        });
    });
}

struct Button {
    label: String,
}

impl Button {
    fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl FlexWidget for Button {
    type Response = Response;

    fn flex_ui(self, item: FlexItem, flex_instance: &mut FlexInstance) -> Self::Response {
        flex_instance
            .add_ui(
                item.sense(Sense::click())
                    .min_height(60.0)
                    .frame_builder(|ui, response| {
                        let style = ui.style().interact(response);

                        (
                            Frame::NONE.fill(style.bg_fill).stroke(style.bg_stroke),
                            TSTransform::default(),
                        )
                    }),
                |ui| {
                    ui.add(Label::new(self.label.clone()));
                },
            )
            .response
    }
}
