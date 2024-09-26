use crate::form::FormFieldState;
use crate::validation_report::IntoFieldPath;
use crate::{EguiValidationReport, Form};
use egui::{Response, RichText, TextStyle, Widget};
use std::borrow::Cow;

/// A form field that can be validated.
/// Will color the field red (using the color from [`egui::style::Visuals::error_fg_color`]) if there is an error.
/// Will show the error message below the field if the field is blurred and there is an error.
pub struct FormField<'a, 'f, Errors: EguiValidationReport> {
    error: Option<Cow<'static, str>>,
    label: Option<Cow<'a, str>>,
    form: Option<&'f mut Form<Errors>>,
}

impl<'a, 'f, Errors: EguiValidationReport> FormField<'a, 'f, Errors> {
    /// Create a new `FormField`.
    /// Pass a [Form] and a reference to the field you want to validate.
    /// If you use [`crate::garde`], just pass the field name / path as a string.
    /// If you use [`crate::validator`], pass a field reference using the [`crate::field_path`] macro.
    pub fn new<'c, I: IntoFieldPath<Errors::FieldPath<'c>>>(
        form: &'f mut Form<Errors>,
        into_field_path: I,
    ) -> Self {
        let field_path = into_field_path.into_field_path();
        let error = form
            .validation_results
            .iter()
            .find_map(|errors| errors.get_field_error(field_path.clone()));

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
                let mut rich_text = RichText::new(label);
                if show_error {
                    rich_text = rich_text.color(error_color);
                }
                ui.label(
                    rich_text.size(
                        ui.style()
                            .text_styles
                            .get(&TextStyle::Body)
                            .map_or(16.0, |s| s.size)
                            * 0.9,
                    ),
                );
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
