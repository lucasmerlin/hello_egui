use crate::EguiValidationReport;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub use crate::_garde_field_path as field_path;
use crate::validation_report::IntoFieldPath;
pub use garde;
use garde::Path;

/// Create a [`garde::Path`] to be submitted to [`crate::FormField::new`]
/// Example:
/// ```rust
/// use egui_form::garde::field_path;
/// use garde::Path;
/// assert_eq!(
///     field_path!("root", "vec", 0, "nested"),
///     Path::new("root").join("vec").join(0).join("nested")
/// )
/// ```
#[macro_export]
macro_rules! _garde_field_path {
    (
        $($field:expr $(,)?)+
    ) => {
        $crate::garde::garde::Path::empty()
        $(
            .join($field)
        )+
    };
}

/// A wrapper around a [`garde::Report`] that implements [`EguiValidationReport`].
pub struct GardeReport(BTreeMap<garde::Path, garde::Error>);

impl GardeReport {
    /// Create a new [`GardeReport`] from a [`garde::Report`].
    /// You can call this function with the result of a call to [`garde::Validate::validate`].
    ///
    /// # Example
    /// ```
    /// use egui_form::garde::{field_path, GardeReport};
    /// use egui_form::{EguiValidationReport, IntoFieldPath};
    /// use garde::Validate;
    /// #[derive(Validate)]
    /// struct Test {
    ///     #[garde(length(min = 3, max = 10))]
    ///     pub user_name: String,
    ///     #[garde(inner(length(min = 3, max = 10)))]
    ///     pub tags: Vec<String>,
    /// }
    ///
    /// let test = Test {
    ///     user_name: "testfiwuehfwoi".to_string(),
    ///     tags: vec!["tag1".to_string(), "waaaaytooooloooong".to_string()],
    /// };
    ///
    /// let report = GardeReport::new(test.validate());
    ///
    /// assert!(report
    ///     .get_field_error(field_path!("user_name").into_field_path())
    ///     .is_some());
    /// assert!(report
    ///     .get_field_error(field_path!("tags", 1).into_field_path())
    ///     .is_some());
    /// ```
    pub fn new(result: Result<(), garde::Report>) -> Self {
        if let Err(errors) = result {
            GardeReport(errors.iter().cloned().collect())
        } else {
            GardeReport(BTreeMap::new())
        }
    }
}

impl EguiValidationReport for GardeReport {
    type FieldPath<'a> = Path;
    type Errors = BTreeMap<Path, garde::Error>;

    fn get_field_error(&self, field: Self::FieldPath<'_>) -> Option<Cow<'static, str>> {
        self.0.get(&field).map(|e| e.to_string().into())
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

impl IntoFieldPath<Path> for Path {
    fn into_field_path(self) -> Path {
        self
    }
}

impl IntoFieldPath<Path> for &str {
    fn into_field_path(self) -> Path {
        Path::new(self)
    }
}

impl IntoFieldPath<Path> for String {
    fn into_field_path(self) -> Path {
        Path::new(self)
    }
}

impl IntoFieldPath<Path> for usize {
    fn into_field_path(self) -> Path {
        Path::new(self)
    }
}
