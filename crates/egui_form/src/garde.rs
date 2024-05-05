use crate::EguiValidationReport;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub use garde;

/// A wrapper around a [garde::Report] that implements [EguiValidationReport].
pub struct GardeReport(BTreeMap<String, garde::Error>);

impl GardeReport {
    /// Create a new [GardeReport] from a [garde::Report].
    /// You can call this function with the result of a call to [garde::Validate::validate].
    ///
    /// # Example
    /// ```
    /// use garde::Validate;
    /// use egui_form::EguiValidationReport;
    /// #[derive(Validate)]
    /// struct Test {
    ///    #[garde(length(min = 3, max = 10))]
    ///   pub user_name: String,
    ///   #[garde(inner(length(min = 3, max = 10)))]
    ///   pub tags: Vec<String>,
    /// }
    ///
    /// let test = Test {
    ///    user_name: "testfiwuehfwoi".to_string(),
    ///    tags: vec!["tag1".to_string(), "waaaaytooooloooong".to_string()],
    /// };
    ///
    /// let report = egui_form::garde::GardeReport::new(test.validate(&()));
    ///
    /// assert!(report.get_field_error("user_name").is_some());
    /// assert!(report.get_field_error("tags[1]").is_some());
    /// ```
    pub fn new(result: Result<(), garde::Report>) -> Self {
        if let Err(errors) = result {
            GardeReport(
                errors
                    .iter()
                    .map(|(path, error)| (path.to_string(), error.clone()))
                    .collect(),
            )
        } else {
            GardeReport(BTreeMap::new())
        }
    }
}

impl EguiValidationReport for GardeReport {
    type FieldPath<'a> = &'a str;
    type Errors = BTreeMap<String, garde::Error>;

    fn get_field_error(&self, field: Self::FieldPath<'_>) -> Option<Cow<'static, str>> {
        self.0.get(field).map(|e| e.to_string().into())
    }

    fn has_errors(&self) -> bool {
        !self.0.is_empty()
    }

    fn error_count(&self) -> usize {
        self.0.len()
    }

    fn get_errors(&self) -> Option<&Self::Errors> {
        if self.has_errors() {
            Some(&self.0)
        } else {
            None
        }
    }
}
