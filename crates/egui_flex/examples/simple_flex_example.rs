use eframe::NativeOptions;
use egui::{CentralPanel, Label, TextEdit, Widget};
use egui_flex::{Flex, FlexAlignContent, FlexItem};

fn main() -> eframe::Result {
    let mut text = "Hello, World!".to_owned();

    eframe::run_simple_native(file!(), NativeOptions::default(), move |ctx, _frame| {
        CentralPanel::default().show(ctx, |ui| {
            // Flex::horizontal().show(ui, |flex| {
            //     flex.add_container(item().grow(1.0), |ui, container| {
            //         ui.scope(|ui| {
            //             Frame::none()
            //                 .stroke(Stroke::new(1.0, Color32::RED))
            //                 .show(ui, |ui| {
            //                     ui.set_min_height(100.0);
            //
            //                     container.content_flex(
            //                         ui,
            //                         Flex::horizontal().align_content(FlexAlignContent::Stretch),
            //                         |flex| {
            //                             flex.add(
            //                                 item().grow(1.0).basis(30.0).align_self(FlexAlign::End),
            //                                 TextEdit::multiline(&mut text),
            //                             );
            //                         },
            //                     )
            //                 })
            //                 .inner
            //         })
            //         .inner
            //     });
            // });

            Flex::horizontal()
                .align_content(FlexAlignContent::Stretch)
                .w_full()
                .show(ui, |flex| {
                    // flex.add(FlexItem::new(), TextEdit::singleline(&mut text));
                    flex.add_ui(FlexItem::new().grow(1.0), |ui| {
                        TextEdit::singleline(&mut text).ui(ui)
                    });
                    flex.add_ui(FlexItem::new().grow(1.0), |ui| {
                        Label::new("Send\nMultiline").ui(ui)
                    });
                });

            // Flex::horizontal().show(ui, |flex| {
            //     flex.add_widget(item().grow(1.0), TextEdit::singleline(&mut text));
            //
            //     // flex.add(item(), Button::new("Send\nMultiline"));
            //     flex.add(item(), Label::new("Send\nMultilinee"));
            //
            //     flex.add(item(), Button::new("Send"));
            // });

            // Flex::horizontal().show(ui, |flex| {
            //     flex.add(item(), Button::new("Non-growing button"));
            //
            //     // Nested flex
            //     flex.add_flex(
            //         item().grow(1.0),
            //         // We need the FlexAlignContent::Stretch to make the buttons fill the space
            //         Flex::vertical().align_content(FlexAlignContent::Stretch),
            //         |flex| {
            //             flex.add(item(), Button::new("Vertical button"));
            //             flex.add(item(), Button::new("Another Vertical button"));
            //         },
            //     );
            // });
            //
            // Flex::horizontal().show(ui, |flex| {
            //     flex.add_container(item().grow(1.0), |ui, container| {
            //         ui.set_min_height(100.0);
            //         container.content_flex(
            //             ui,
            //             Flex::horizontal().align_content(FlexAlignContent::Stretch),
            //             |flex| {
            //                 flex.add_container(
            //                     item().grow(1.0).align_self(FlexAlign::Center),
            //                     |ui, container| {
            //                         container.content(ui, |ui| {
            //                             TextEdit::singleline(&mut String::new())
            //                                 .desired_width(ui.available_width())
            //                                 .ui(ui)
            //                         })
            //                     },
            //                 )
            //             },
            //         )
            //     });
            // })
        });
    })
}
