use eframe::NativeOptions;
use egui::{vec2, TextEdit, Widget, Window};

use egui_webview::native_text_field::{NativeTextField, TextFieldType};
use egui_webview::{init_webview, webview_end_frame};

pub fn main() -> eframe::Result<()> {
    let mut view = None;

    eframe::run_simple_native(
        "Dnd Example App",
        NativeOptions::default(),
        move |ctx, frame| {
            if view.is_none() {
                init_webview(ctx);

                view = Some((
                    NativeTextField::new(ctx, "email", TextFieldType::Email, frame),
                    NativeTextField::new(ctx, "password", TextFieldType::Password, frame),
                    NativeTextField::new(ctx, "textarea", TextFieldType::Textarea, frame),
                ));
            }

            let view = view.as_mut().unwrap();

            Window::new("Hello").show(ctx, |ui| {
                ui.label("Email");
                view.0.ui(ui, vec2(ui.available_size().x, 20.0));

                ui.add_space(8.0);
                ui.label("Password");
                view.1.ui(ui, vec2(ui.available_size().x, 20.0));

                ui.add_space(8.0);
                ui.label("Textarea");
                view.2.ui(ui, vec2(ui.available_size().x, 100.0));
            });

            Window::new("Hello 2").show(ctx, |ui| {
                ui.label("Email");
                view.0
                    .current_text_mut(|text| ui.text_edit_singleline(text).changed());

                ui.label("Password");
                view.1.current_text_mut(|text| {
                    TextEdit::singleline(text).password(true).ui(ui).changed()
                });

                ui.label("Textarea");
                view.2
                    .current_text_mut(|text| TextEdit::multiline(text).ui(ui).changed());
            });

            webview_end_frame(ctx);
        },
    )
}
