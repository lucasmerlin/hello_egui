use eframe::NativeOptions;
use egui::CentralPanel;
use egui_form::validator::field_path;
use egui_form::Form;
use egui_form::FormField;
use validator::Validate;

#[derive(Validate, Debug)]
struct Test {
    #[validate(length(min = 3, max = 10))]
    pub user_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(nested)]
    pub nested: Nested,
    #[validate(nested)]
    pub vec: Vec<Nested>,
}

#[derive(Validate, Debug)]
struct Nested {
    #[validate(range(
        min = 1,
        max = 10,
        message = "Custom Message: Must be between 1 and 10"
    ))]
    pub test: u64,
}

fn form_ui(ui: &mut egui::Ui, test: &mut Test) {
    let mut form = Form::new().add_report(
        egui_form::validator::ValidatorReport::new(test.validate()).with_translation(|error| {
            // Since validator doesn't have default messages, we have to provide our own
            if let Some(msg) = &error.message {
                return msg.clone();
            }

            match error.code.as_ref() {
                "email" => "Invalid email".into(),
                "length" => format!(
                    "Must be between {} and {} characters long",
                    error.params["min"], error.params["max"]
                )
                .into(),
                _ => format!("Validation Failed: {}", error.code).into(),
            }
        }),
    );

    FormField::new(&mut form, "user_name")
        .label("User Name")
        .ui(ui, egui::TextEdit::singleline(&mut test.user_name));
    FormField::new(&mut form, "email")
        .label("Email")
        .ui(ui, egui::TextEdit::singleline(&mut test.email));
    FormField::new(&mut form, field_path!("nested", "test"))
        .label("Nested Test")
        .ui(ui, egui::Slider::new(&mut test.nested.test, 0..=11));
    FormField::new(&mut form, field_path!("vec", 0, "test"))
        .label("Vec Test")
        .ui(
            ui,
            egui::DragValue::new(&mut test.vec[0].test).range(0..=11),
        );

    if let Some(Ok(())) = form.handle_submit(&ui.button("Submit"), ui) {
        println!("Form submitted: {test:?}");
    }
}

fn main() -> eframe::Result<()> {
    let mut test = Test {
        user_name: "testfiwuehfwoi".to_string(),
        email: "garbage".to_string(),
        nested: Nested { test: 0 },
        vec: vec![Nested { test: 0 }],
    };

    eframe::run_simple_native(
        "Egui Validator Validation",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                form_ui(ui, &mut test);
            });
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_form::validator::field_path;
    use egui_form::{EguiValidationReport, IntoFieldPath};

    #[test]
    fn test_validate() {
        let test = Test {
            user_name: "testfiwuehfwoi".to_string(),
            email: "garbage".to_string(),
            nested: Nested { test: 0 },
            vec: vec![Nested { test: 0 }],
        };

        let report = egui_form::validator::ValidatorReport::validate(test);

        assert!(report
            .get_field_error(field_path!("user_name").into_field_path())
            .is_some());
        assert!(report
            .get_field_error(field_path!("email").into_field_path())
            .is_some());
        assert!(report
            .get_field_error(field_path!("nested", "test").into_field_path())
            .is_some());
        assert!(report
            .get_field_error(field_path!("vec", 0, "test").into_field_path())
            .is_some());

        assert_eq!(report.error_count(), 4);
    }
}
