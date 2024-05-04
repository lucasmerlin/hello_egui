use eframe::NativeOptions;
use egui::CentralPanel;
use egui::WidgetType::TextEdit;
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

fn main() -> eframe::Result<()> {
    let mut test = Test {
        user_name: "testfiwuehfwoi".to_string(),
        email: "lefwojwfpke".to_string(),
        nested: Nested { test: 0 },
        vec: vec![Nested { test: 0 }],
    };

    let result = test.validate(&());

    if let Err(report) = result {
        for (path, error) in report.iter() {
            println!("{}: {}", path, error);
        }
    }

    eframe::run_simple_native(
        "Egui Garde Validation",
        NativeOptions::default(),
        move |ctx, frame| {
            CentralPanel::default().show(ctx, |ui| {
                let mut form =
                    Form::new().validate(egui_form::garde::GardeReport::new(test.validate(&())));

                FormField::new(&mut form, "user_name")
                    .label("User Name")
                    .ui(ui, egui::TextEdit::singleline(&mut test.user_name));
                FormField::new(&mut form, "email")
                    .label("Email")
                    .ui(ui, egui::TextEdit::singleline(&mut test.email));
                FormField::new(&mut form, "nested.test")
                    .label("Nested Test")
                    .ui(ui, egui::Slider::new(&mut test.nested.test, 0..=11));
                FormField::new(&mut form, "vec[0].test")
                    .label("Vec Test")
                    .ui(
                        ui,
                        egui::DragValue::new(&mut test.vec[0].test).clamp_range(0..=11),
                    );

                if form.handle_submit(&ui.button("Submit"), ui) {
                    println!("Form submitted: {:?}", test);
                }
            });
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_form::garde::GardeReport;
    use egui_form::EguiValidationErrors;

    #[test]
    fn test() {
        let test = Test {
            user_name: "testfiwuehfwoi".to_string(),
            email: "garbage".to_string(),
            nested: Nested { test: 0 },
            vec: vec![Nested { test: 0 }],
        };

        let report = GardeReport::new(test.validate(&()));

        assert!(report.get_field_error("user_name").is_some());
        assert!(report.get_field_error("email").is_some());
        assert!(report.get_field_error("nested.test").is_some());
        assert!(report.get_field_error("vec[0].test").is_some());

        assert_eq!(report.error_count(), 4);
    }
}
