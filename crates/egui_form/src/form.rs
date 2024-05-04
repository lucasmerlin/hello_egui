use crate::EguiValidationErrors;
use egui::{Response, Ui};

pub(crate) struct FormFieldState {
    pub(crate) state_id: egui::Id,
    pub(crate) widget_id: egui::Id,
    // TODO: I don't think this is needed anymore
    pub(crate) errors: Vec<String>,
}

pub struct Form<R: EguiValidationErrors> {
    pub(crate) controls: Vec<FormFieldState>,
    pub(crate) validation_results: Vec<R>,
}

impl<R: EguiValidationErrors> Default for Form<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: EguiValidationErrors> Form<R> {
    pub fn new() -> Self {
        Self {
            controls: Vec::new(),
            validation_results: Vec::new(),
        }
    }

    pub fn validate(mut self, value: R) -> Self {
        self.validation_results.push(value);
        self
    }

    pub fn handle_submit(&mut self, response: &Response, ui: &mut Ui) -> bool {
        if response.clicked() {
            let has_errors = self
                .controls
                .iter()
                .any(|control| !control.errors.is_empty());
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
                false
            } else {
                true
            }
        } else {
            false
        }
    }
}
