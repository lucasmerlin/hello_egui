use eframe::NativeOptions;
use egui::CentralPanel;

use egui_form::garde::field_path;
use egui_form::{Form, FormField};
use garde::Validate;

#[derive(Validate, Debug)]
struct Test {
    #[garde(length(min = 3, max = 10))]
    pub user_name: String,
    #[garde(email)]
    pub email: String,
    #[garde(dive)]
    pub nested: Nested,
    #[garde(dive)]
    pub vec: Vec<Nested>,
}

#[derive(Validate, Debug)]
struct Nested {
    #[garde(range(min = 1, max = 10))]
    pub test: u64,
}

fn form_ui(ui: &mut egui::Ui, test: &mut Test) {
    let mut form = Form::new().add_report(egui_form::garde::GardeReport::new(test.validate()));

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
        email: "lefwojwfpke".to_string(),
        nested: Nested { test: 0 },
        vec: vec![Nested { test: 0 }],
    };

    eframe::run_simple_native(
        "Egui Garde Validation",
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
    use egui_form::garde::GardeReport;
    use egui_form::{EguiValidationReport, IntoFieldPath};

    #[test]
    fn test() {
        let test = Test {
            user_name: "testfiwuehfwoi".to_string(),
            email: "garbage".to_string(),
            nested: Nested { test: 0 },
            vec: vec![Nested { test: 0 }],
        };

        let report = GardeReport::new(test.validate());

        assert!(report
            .get_field_error("user_name".into_field_path())
            .is_some());
        assert!(report.get_field_error(field_path!("email")).is_some());
        assert!(report
            .get_field_error(field_path!("nested", "test"))
            .is_some());
        assert!(report
            .get_field_error(field_path!("vec", 0, "test"))
            .is_some());

        assert_eq!(report.error_count(), 4);
    }
}
