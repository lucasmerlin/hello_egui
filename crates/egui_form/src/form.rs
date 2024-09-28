use crate::EguiValidationReport;
use egui::{Response, Ui};

pub(crate) struct FormFieldState {
    pub(crate) state_id: egui::Id,
    pub(crate) widget_id: egui::Id,
    // TODO: I don't think this is needed anymore
    pub(crate) errors: Vec<String>,
}

/// Form connects the state of the individual form fields with the validation results.
/// It's also responsible for handling the submission and focusing the first invalid field on error.
pub struct Form<R: EguiValidationReport> {
    pub(crate) controls: Vec<FormFieldState>,
    pub(crate) validation_results: Vec<R>,
}

impl<R: EguiValidationReport> Default for Form<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: EguiValidationReport> Form<R> {
    /// Create a new form.
    pub fn new() -> Self {
        Self {
            controls: Vec::new(),
            validation_results: Vec::new(),
        }
    }

    /// Add a validation report to the form.
    /// This will be either a [`crate::validator::ValidatorReport`] or a [`crate::garde::GardeReport`].
    /// You can add multiple reports to the form.
    /// You can also pass a custom Report for your own validation logic, if you implement [`EguiValidationReport`] for it.
    pub fn add_report(mut self, value: R) -> Self {
        self.validation_results.push(value);
        self
    }

    /// Handle the submission of the form.
    /// You usually pass this a button response.
    /// If this function returns Some(Ok(_)), the form data can be submitted.
    ///
    /// You can also use [`EguiValidationReport::try_submit`] directly.
    pub fn handle_submit(
        &mut self,
        response: &Response,
        ui: &mut Ui,
    ) -> Option<Result<(), Vec<&R::Errors>>> {
        if response.clicked() {
            Some(self.try_submit(ui))
        } else {
            None
        }
    }

    /// Try to submit the form.
    /// Returns Ok(()) if the form is valid, otherwise returns the errors.
    pub fn try_submit(&mut self, ui: &mut Ui) -> Result<(), Vec<&R::Errors>> {
        let has_errors = self
            .validation_results
            .iter()
            .any(super::validation_report::EguiValidationReport::has_errors);
        if has_errors {
            ui.memory_mut(|mem| {
                for control in &self.controls {
                    mem.data.insert_temp(control.state_id, true);
                }
                if let Some(first) = self
                    .controls
                    .iter()
                    .find(|control| !control.errors.is_empty())
                {
                    mem.request_focus(first.widget_id);
                }
            });
            Err(self
                .validation_results
                .iter()
                .filter_map(|e| e.get_errors())
                .collect())
        } else {
            Ok(())
        }
    }
}
