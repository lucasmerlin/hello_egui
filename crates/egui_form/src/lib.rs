#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

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
/// # use egui_form::garde::field_path;
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
///     let mut form =
///         Form::new().add_report(egui_form::garde::GardeReport::new(test.validate(&())));
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
/// ```
#[cfg(feature = "validator_garde")]
pub mod garde;
mod validation_report;

mod form_field;
/// To use [validator] with `egui_form`, you need to create a [`validator::ValidatorReport`] and pass it to the [Form] instance.
///
/// Then, when you create a [`FormField`], you pass a slice of [`validator::PathItem`]s.
/// Usually, you would use the [`field_path`!] macro to create the slice.
/// For nested fields and arrays, the syntax for the field name looks like this:
/// `field_path!("nested", "array", 0, "field")`
///
/// # Example
/// ```no_run
/// # // Taken 1:1 from crates/egui_form/examples/validator.rs
/// # use eframe::NativeOptions;
/// # use egui::CentralPanel;
/// # use egui_form::{Form, FormField};
/// # use validator::Validate;
/// # use egui_form::validator::field_path;
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

pub use form::Form;
pub use form_field::*;
pub use validation_report::{EguiValidationReport, IntoFieldPath};
