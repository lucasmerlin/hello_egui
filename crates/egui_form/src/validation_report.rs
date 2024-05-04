use std::borrow::Cow;

/// A trait telling egui_form how to parse validation errors.
pub trait EguiValidationReport {
    /// The type used to identify fields.
    type FieldPath<'a>: Copy;
    /// The type of the errors.
    type Errors;

    /// Returns the error message for a field.
    fn get_field_error(&self, field: Self::FieldPath<'_>) -> Option<Cow<'static, str>>;

    /// Returns true if there are any errors.
    fn has_errors(&self) -> bool;

    /// Returns the number of errors.
    fn error_count(&self) -> usize;

    /// Returns a reference to the errors.
    fn get_errors(&self) -> Option<&Self::Errors>;
}
