use eframe::NativeOptions;
use egui::{TextEdit, Ui};
use egui_form::garde::GardeReport;
use egui_form::{Form, FormField};
use garde::Validate;

#[derive(Debug, Default, Validate)]
struct Fields {
    #[garde(length(min = 2, max = 50))]
    user_name: String,
}

fn form_ui(ui: &mut Ui, fields: &mut Fields) {
    let mut form = Form::new().add_report(GardeReport::new(fields.validate(&())));

    FormField::new(&mut form, "user_name")
        .label("User Name")
        .ui(ui, TextEdit::singleline(&mut fields.user_name));

    if let Some(Ok(())) = form.handle_submit(&ui.button("Submit"), ui) {
        println!("Submitted: {:?}", fields);
    }
}

fn main() -> eframe::Result<()> {
    let mut fields = Fields::default();

    eframe::run_simple_native(
        "egui_form minimal example",
        NativeOptions::default(),
        move |ctx, _frame| {
            egui::CentralPanel::default().show(ctx, |ui| {
                form_ui(ui, &mut fields);
            });
        },
    )
}
