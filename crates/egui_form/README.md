# egui_form

[![egui_ver](https://img.shields.io/badge/egui-0.30.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_form.svg)](https://crates.io/crates/egui_form)
[![Documentation](https://docs.rs/egui_form/badge.svg)](https://docs.rs/egui_form)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_form.svg)](https://crates.io/crates/egui_form)



[content]:<>


egui_form adds form validation to egui.
It can either use [validator](https://crates.io/crates/validator)
or [garde](https://crates.io/crates/garde) for validation.
This also means, if you use rust you can use the same validation logic
on the server and the client.

Check the docs for the [validator implementation](https://docs.rs/egui_form/latest/egui_form/validator/index.html)
or the [garde implementation](https://docs.rs/egui_form/latest/egui_form/garde/index.html)
to get started.

You can also build a custom implementation by implementing the `EguiValidationReport` for the result of whatever
form validation crate you use.

## Showcase

You can [try the Signup Form example](https://lucasmerlin.github.io/hello_egui/#/example/signup_form) in hello_egui
showcase app.

Also, here's a screenshot from HelloPaint's profile form:

![screenshot](https://github.com/lucasmerlin/hello_egui/blob/main/crates/egui_form/screenshot.png?raw=true)

## Should I use validator or garde?

For small / prototype projects, I'd recommend garde, since it has built in error messages.
For bigger projects that might require i18n, it might make sense to use validator,
since it allows for custom error messages (garde as of now has no i18n support).

In HelloPaint I'm using garde, since it seems a bit cleaner and more active, hoping
that i18n will be solved before it becomes a problem for HelloPaint.

## Minimal example using garde

From [egui_form_minimal.rs](https://github.com/lucasmerlin/hello_egui/blob/main/crates/egui_form/examples/egui_form_minimal.rs)

```rust
use eframe::NativeOptions;
use egui::{TextEdit, Ui};
use egui_form::garde::{GardeReport, field_path};
use egui_form::{Form, FormField};
use garde::Validate;


#[derive(Debug, Default, Validate)]
struct Fields {
    #[garde(length(min = 2, max = 50))]
    user_name: String,
}

fn form_ui(ui: &mut Ui, fields: &mut Fields) {
    let mut form = Form::new().add_report(GardeReport::new(fields.validate()));

    FormField::new(&mut form, field_path!("user_name"))
        .label("User Name")
        .ui(ui, TextEdit::singleline(&mut fields.user_name));

    if let Some(Ok(())) = form.handle_submit(&ui.button("Submit"), ui) {
        println!("Submitted: {:?}", fields);
    }
}
```
