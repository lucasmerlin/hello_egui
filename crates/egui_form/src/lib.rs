#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::{Response, RichText, Widget};
use std::borrow::Cow;

mod form;

/// To use [garde] with egui_form, you need to create a [garde::GardeReport] and pass it to the [Form] instance.
///
/// Then, when you create a [FormField], you pass the field's name as a &str.
/// For nested fields and arrays, the syntax for the field name looks like this:
/// `nested.array[0].field`
///
/// # Example
/// ```no_run
/// # use eframe::NativeOptions;
/// # use egui::CentralPanel;
/// #
/// # use egui_form::{Form, FormField};
/// # use garde::Validate;
///
/// #[derive(Validate, Debug)]
/// struct Test {
///     #[garde(length(min = 3, max = 10))]
///     pub user_name: String,
///     #[garde(email)]
///     pub email: String,
///     #[garde(dive)]
///     pub nested: Nested,
///     #[garde(dive)]
///     pub vec: Vec<Nested>,
/// }
///
/// #[derive(Validate, Debug)]
/// struct Nested {
///     #[garde(range(min = 1, max = 10))]
///     pub test: u64,
/// }
///
/// pub fn form_ui(ui: &mut egui::Ui, test: &mut Test) {
///     let mut form = Form::new().add_report(egui_form::garde::GardeReport::new(test.validate(&())));
///
///     FormField::new(&mut form, "user_name")
///         .label("User Name")
///         .ui(ui, egui::TextEdit::singleline(&mut test.user_name));
///     FormField::new(&mut form, "email")
///         .label("Email")
///         .ui(ui, egui::TextEdit::singleline(&mut test.email));
///     FormField::new(&mut form, "nested.test")
///         .label("Nested Test")
///         .ui(ui, egui::Slider::new(&mut test.nested.test, 0..=11));
///     FormField::new(&mut form, "vec[0].test")
///         .label("Vec Test")
///         .ui(
///             ui,
///             egui::DragValue::new(&mut test.vec[0].test).clamp_range(0..=11),
///         );
///
///     if let Some(Ok(())) = form.handle_submit(&ui.button("Submit"), ui) {
///         println!("Form submitted: {:?}", test);
///     }
/// }
/// ```
#[cfg(feature = "validator_garde")]
pub mod garde;
mod validation_report;

/// To use [validator] with egui_form, you need to create a [validator::ValidatorReport] and pass it to the [Form] instance.
///
/// Then, when you create a [FormField], you pass a slice of [validator::PathItem]s.
/// Usually, you would use the [field_path!] macro to create the slice.
/// For nested fields and arrays, the syntax for the field name looks like this:
/// `field_path!("nested", "array", 0, "field")`
///
/// # Example
/// ```no_run
/// # // Taken 1:1 from crates/egui_form/examples/validator.rs
/// # use eframe::NativeOptions;
/// # use egui::CentralPanel;
/// # use egui_form::{field_path, Form, FormField};
/// # use validator::Validate;
///
/// #[derive(Validate, Debug)]
/// struct Test {
///     #[validate(length(min = 3, max = 10))]
///     pub user_name: String,
///     #[validate(email)]
///     pub email: String,
///     #[validate(nested)]
///     pub nested: Nested,
///     #[validate(nested)]
///     pub vec: Vec<Nested>,
/// }
///
/// #[derive(Validate, Debug)]
/// struct Nested {
///     #[validate(range(
///         min = 1,
///         max = 10,
///         message = "Custom Message: Must be between 1 and 10"
///     ))]
///     pub test: u64,
/// }
///
/// fn form_ui(ui: &mut egui::Ui, test: &mut Test) {
///     let mut form = Form::new().add_report(
///         egui_form::validator::ValidatorReport::new(test.validate()).with_translation(|error| {
///             // Since validator doesn't have default messages, we have to provide our own
///             if let Some(msg) = &error.message {
///                 return msg.clone();
///             }
///
///             match error.code.as_ref() {
///                 "email" => "Invalid email".into(),
///                 "length" => format!(
///                     "Must be between {} and {} characters long",
///                     error.params["min"], error.params["max"]
///                 )
///                 .into(),
///                 _ => format!("Validation Failed: {}", error.code).into(),
///             }
///         }),
///     );
///
///     FormField::new(&mut form, field_path!("user_name"))
///         .label("User Name")
///         .ui(ui, egui::TextEdit::singleline(&mut test.user_name));
///     FormField::new(&mut form, field_path!("email"))
///         .label("Email")
///         .ui(ui, egui::TextEdit::singleline(&mut test.email));
///     FormField::new(&mut form, field_path!("nested", "test"))
///         .label("Nested Test")
///         .ui(ui, egui::Slider::new(&mut test.nested.test, 0..=11));
///     FormField::new(&mut form, field_path!("vec", 0, "test"))
///         .label("Vec Test")
///         .ui(
///             ui,
///             egui::DragValue::new(&mut test.vec[0].test).clamp_range(0..=11),
///         );
///
///     if let Some(Ok(())) = form.handle_submit(&ui.button("Submit"), ui) {
///         println!("Form submitted: {:?}", test);
///     }
/// }
#[cfg(feature = "validator_validator")]
pub mod validator;

use crate::form::FormFieldState;
pub use form::Form;
pub use validation_report::EguiValidationReport;

/// A form field that can be validated.
/// Will color the field red (using the color from [egui::style::Visuals]::error_fg_color) if there is an error.
/// Will show the error message below the field if the field is blurred and there is an error.
pub struct FormField<'a, 'f, Errors: EguiValidationReport> {
    error: Option<Cow<'static, str>>,
    label: Option<Cow<'a, str>>,
    form: Option<&'f mut Form<Errors>>,
}

impl<'a, 'f, Errors: EguiValidationReport> FormField<'a, 'f, Errors> {
    /// Create a new FormField.
    /// Pass a [form::Form] and a reference to the field you want to validate.
    /// If you use [garde], just pass the field name / path as a string.
    /// If you use [validator], pass a field reference using the [crate::field_path] macro.
    pub fn new<'c>(form: &'f mut form::Form<Errors>, field: Errors::FieldPath<'c>) -> Self {
        let error = form
            .validation_results
            .iter()
            .find_map(|errors| errors.get_field_error(field));

        FormField {
            error,
            label: None,
            form: Some(form),
        }
    }

    /// Optionally set a label for the field.
    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Render the field.
    pub fn ui(self, ui: &mut egui::Ui, content: impl Widget) -> Response {
        let error = self.error;

        ui.vertical(|ui| {
            let id = ui.auto_id_with("form_field");
            let blurred = ui.memory_mut(|mem| *mem.data.get_temp_mut_or(id, false));

            let error_color = ui.style().visuals.error_fg_color;

            let show_error = error.is_some() && blurred;

            if show_error {
                let widgets = &mut ui.style_mut().visuals.widgets;
                widgets.inactive.bg_stroke.color = error_color;
                widgets.inactive.bg_stroke.width = 1.0;
                widgets.active.bg_stroke.color = error_color;
                widgets.active.bg_stroke.width = 1.0;
                widgets.hovered.bg_stroke.color = error_color;
                widgets.hovered.bg_stroke.width = 1.0;
                widgets.open.bg_stroke.color = error_color;
                widgets.open.bg_stroke.width = 1.0;
            }

            if let Some(label) = self.label {
                let mut rich_text = RichText::new(label.as_ref());
                if show_error {
                    rich_text = rich_text.color(error_color);
                }
                ui.label(rich_text);
            }

            let response = content.ui(ui);

            if response.lost_focus() {
                ui.memory_mut(|mem| {
                    mem.data.insert_temp(id, true);
                });
            }

            if let Some(form) = self.form {
                if let Some(error) = &error {
                    form.controls.push(FormFieldState {
                        state_id: id,
                        widget_id: response.id,
                        errors: vec![error.to_string()],
                    });
                } else {
                    form.controls.push(FormFieldState {
                        state_id: id,
                        widget_id: response.id,
                        errors: vec![],
                    });
                };
            }

            ui.add_visible(
                show_error,
                egui::Label::new(
                    RichText::new(error.as_deref().unwrap_or(""))
                        .color(error_color)
                        .small(),
                ),
            );

            response
        })
        .inner
    }
}
