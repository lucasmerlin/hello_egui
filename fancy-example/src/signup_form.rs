use crate::crate_ui::{crate_usage_ui, Crate, CrateUsage};
use crate::shared_state::SharedState;
use crate::sidebar::Example;
use crate::{crate_usage, demo_area};

use egui::{vec2, Button, ScrollArea, TextEdit, Ui, Widget};
use egui_extras::Size;
use egui_form::validator::validator::Validate;
use egui_form::validator::ValidatorReport;
use egui_form::{field_path, Form, FormField};
use validator::{ValidateArgs, ValidationError};

fn validate_password(password: &str) -> Result<(), ValidationError> {
    if password.chars().all(|c| c.is_alphanumeric()) {
        return Err(ValidationError::new("validate_password"));
    }
    if password.chars().all(|c| c.is_lowercase()) {
        return Err(ValidationError::new("validate_password"));
    }
    if password.chars().all(|c| c.is_uppercase()) {
        return Err(ValidationError::new("validate_password"));
    }
    if password.chars().all(|c| !c.is_ascii_digit()) {
        return Err(ValidationError::new("validate_password"));
    }
    Ok(())
}

struct ValidateContext<'a> {
    password: &'a str,
}

fn validate_repeat_password(
    repeat_password: &str,
    context: &ValidateContext,
) -> Result<(), ValidationError> {
    if repeat_password != context.password {
        Err(ValidationError::new("password_mismatch"))
    } else {
        Ok(())
    }
}

#[derive(Debug, Default, Validate)]
#[validate(context = "ValidateContext<'v_a>")]
struct SignupFields {
    #[validate(length(min = 2, max = 50))]
    first_name: String,
    #[validate(length(min = 2, max = 50))]
    last_name: String,
    #[validate(email)]
    email: String,
    #[validate(length(min = 8), custom(function = "validate_password"))]
    password: String,
    #[validate(
        length(min = 8),
        custom(function = "validate_repeat_password", use_context)
    )]
    repeat_password: String,
    #[validate(range(min = 18, max = 120, message = "You must be at least 18 years old"))]
    age: u8,
    #[validate(required(message = "You must agree to the terms"))]
    terms: Option<bool>,
    newsletter: bool,
}

#[derive(Default, Debug)]
pub struct SignupForm {
    fields: SignupFields,
    submitted: bool,
}

impl SignupForm {
    pub fn new() -> Self {
        Self {
            fields: SignupFields::default(),
            submitted: false,
        }
    }
}

impl Example for SignupForm {
    fn name(&self) -> &'static str {
        "Signup Form"
    }

    crate_usage!(CrateUsage::simple(Crate::EguiForm));

    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        demo_area(ui, self.name(), 400.0, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.style_mut().spacing.text_edit_width = 400.0;

                if self.submitted {
                    ui.heading("Thank you for signing up!");
                    ui.label("Here are the details you submitted:");
                    ui.code(format!("{:#?}", self.fields));

                    ui.add_space(8.0);

                    if ui.button("Reset").clicked() {
                        self.submitted = false;
                        self.fields = SignupFields::default();
                    }
                    return;
                }

                let mut form = Form::new().validate(
                    ValidatorReport::new(self.fields.validate_with_args(&ValidateContext {
                        password: &self.fields.password,
                    }))
                    .with_translation(|error| {
                        // validator has no default error messages, so we provide our own
                        match error.code.as_ref() {
                            "length" => match (error.params.get("min"), error.params.get("max")) {
                                (Some(min), Some(max)) => {
                                    format!("Must be between {} and {} characters long", min, max)
                                        .into()
                                }
                                (Some(min), None) => {
                                    format!("Must be at least {} characters long", min).into()
                                }
                                (None, Some(max)) => {
                                    format!("Must be at most {} characters long", max).into()
                                }
                                _ => "Invalid length".into(),
                            },
                            "email" => "Invalid email".into(),
                            "range" => format!(
                                "Must be between {} and {}",
                                error.params["min"], error.params["max"]
                            )
                            .into(),
                            "required" => "Required".into(),
                            "validate_password" => {
                                "Password must contain at least one uppercase letter, \
                        one lowercase letter, one digit, and one special character"
                                    .into()
                            }
                            "password_mismatch" => "Passwords do not match".into(),
                            _ => format!("Validation Failed: {}", error.code).into(),
                        }
                    }),
                );

                let form_ref = &mut form;
                let SignupFields {
                    first_name,
                    last_name,
                    email,
                    password,
                    repeat_password,
                    age,
                    terms,
                    newsletter,
                } = &mut self.fields;

                let max_width = ui.available_width();

                ui.label(
                    r#"This is a example signup form showcasing egui_form's form validation.
Try signing up with invalid data to see the validation errors.
Errors will show up after editing a field or after trying to submit.
            "#,
                );

                let horizontal_fields =
                    |ui: &mut Ui,
                     form: &mut Form<ValidatorReport>,
                     a: &mut dyn FnMut(&mut Ui, &mut Form<ValidatorReport>),
                     b: &mut dyn FnMut(&mut Ui, &mut Form<ValidatorReport>)| {
                        if max_width > 300.0 {
                            ui.horizontal(|ui| {
                                ui.set_max_width(max_width);
                                let builder = egui_extras::StripBuilder::new(ui)
                                    .sizes(Size::relative(0.5), 2);
                                builder.horizontal(|mut strip| {
                                    strip.cell(|ui| a(ui, form));
                                    strip.cell(|ui| b(ui, form));
                                });
                            });
                        } else {
                            a(ui, form);
                            b(ui, form);
                        }
                    };

                horizontal_fields(
                    ui,
                    form_ref,
                    &mut |ui, form_ref| {
                        FormField::new(form_ref, field_path!("first_name"))
                            .label("First Name")
                            .ui(ui, TextEdit::singleline(first_name));
                    },
                    &mut |ui, form_ref| {
                        FormField::new(form_ref, field_path!("last_name"))
                            .label("Last Name")
                            .ui(ui, TextEdit::singleline(last_name));
                    },
                );

                FormField::new(form_ref, field_path!("email"))
                    .label("Email")
                    .ui(ui, TextEdit::singleline(email));

                horizontal_fields(
                    ui,
                    form_ref,
                    &mut |ui, form_ref| {
                        FormField::new(form_ref, field_path!("password"))
                            .label("Password")
                            .ui(ui, TextEdit::singleline(password).password(true));
                    },
                    &mut |ui, form_ref| {
                        FormField::new(form_ref, field_path!("repeat_password"))
                            .label("Repeat Password")
                            .ui(ui, TextEdit::singleline(repeat_password).password(true));
                    },
                );

                FormField::new(form_ref, field_path!("age"))
                    .label("Age")
                    .ui(ui, egui::DragValue::new(age));

                let mut checked = terms.is_some();
                FormField::new(form_ref, field_path!("terms")).ui(
                    ui,
                    egui::Checkbox::new(&mut checked, "I agree to the terms"),
                );
                *terms = if checked { Some(true) } else { None };

                FormField::new(form_ref, field_path!("newsletter")).ui(
                    ui,
                    egui::Checkbox::new(newsletter, "Subscribe to newsletter"),
                );

                let response = ui.vertical_centered(|ui| {
                    Button::new("Submit").min_size(vec2(max_width, 40.0)).ui(ui)
                });

                if form.handle_submit(&response.inner, ui) {
                    self.submitted = true;
                }

                ui.add_space(8.0);

                crate_usage_ui(ui, self.crates(), shared_state);
            });
        });
    }
}
