use crate::EguiValidationReport;
use std::borrow::Cow;

pub use crate::_validator_field_path as field_path;
use crate::validation_report::IntoFieldPath;

use std::hash::Hash;
pub use validator;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

/// Represents either a field in a struct or a indexed field in a list.
/// Usually created with the [crate::field_path] macro.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PathItem<'a> {
    /// Field in a struct.
    Field(Cow<'a, str>),
    /// Indexed field in a list.
    Indexed(usize),
}

impl From<usize> for PathItem<'_> {
    fn from(value: usize) -> Self {
        PathItem::Indexed(value)
    }
}

impl From<String> for PathItem<'_> {
    fn from(value: String) -> Self {
        PathItem::Field(Cow::Owned(value))
    }
}

impl<'a> From<&'a str> for PathItem<'a> {
    fn from(value: &'a str) -> Self {
        PathItem::Field(Cow::Borrowed(value))
    }
}

/// Create a field path to be submitted to a [crate::FormField::new].
/// This macro takes a list of field names and indexes and returns a slice of [PathItem]s.
/// # Example
/// ```
/// use egui_form::validator::{PathItem, field_path};
/// assert_eq!(field_path!("nested", "array", 0, "field"), &[
///     PathItem::Field("nested".into()),
///     PathItem::Field("array".into()),
///     PathItem::Indexed(0),
///     PathItem::Field("field".into()),
/// ]);
#[macro_export]
macro_rules! _validator_field_path {
    (
        $($field:expr $(,)?)+
    ) => {
        [
            $(
                $crate::validator::PathItem::from($field),
            )+
        ].as_slice()
    };
}

type GetTranslationFn = Box<dyn Fn(&ValidationError) -> Cow<'static, str>>;

/// Contains the validation errors from [validator]
pub struct ValidatorReport {
    get_t: Option<GetTranslationFn>,
    errors: Option<ValidationErrors>,
}

impl ValidatorReport {
    /// Create a new [ValidatorReport] from a [validator::ValidationErrors].
    /// You can call this function with the result of a call to [validator::Validate::validate].
    pub fn new(result: Result<(), ValidationErrors>) -> Self {
        ValidatorReport {
            errors: result.err(),
            get_t: None,
        }
    }

    /// Convenience function to validate a value and create a [ValidatorReport] from it.
    pub fn validate<T: Validate>(value: T) -> Self {
        let result = value.validate();
        Self::new(result)
    }

    /// Add a custom translation function to the report.
    /// Pass a function that takes a [ValidationError] and returns a translated error message.
    pub fn with_translation<F: Fn(&ValidationError) -> Cow<'static, str> + 'static>(
        mut self,
        get_t: F,
    ) -> Self {
        self.get_t = Some(Box::new(get_t));
        self
    }
}

fn get_error_recursively<'a>(
    errors: &'a ValidationErrors,
    fields: &[PathItem],
) -> Option<&'a Vec<ValidationError>> {
    if let Some((field, rest)) = fields.split_first() {
        let field = match field {
            PathItem::Field(field) => field,
            _ => return None,
        };
        match errors.0.get(field.as_ref()) {
            Some(ValidationErrorsKind::Struct(errors)) => get_error_recursively(errors, rest),
            Some(ValidationErrorsKind::List(errors)) => {
                if let Some((PathItem::Indexed(index), rest)) = rest.split_first() {
                    if let Some(errors) = errors.get(index) {
                        get_error_recursively(errors, rest)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Some(ValidationErrorsKind::Field(errors)) => {
                if rest.is_empty() {
                    Some(errors)
                } else {
                    None
                }
            }
            None => None,
        }
    } else {
        None
    }
}

/// Helper enum to allow passing non nested field paths as a &str, without using the field_path!() macro
#[doc(hidden)]
#[derive(Clone)]
pub enum ValidatorPathType<'a> {
    Single(PathItem<'a>),
    Borrowed(&'a [PathItem<'a>]),
}

impl EguiValidationReport for ValidatorReport {
    type FieldPath<'a> = ValidatorPathType<'a>;
    type Errors = ValidationErrors;

    fn get_field_error(&self, into_path: Self::FieldPath<'_>) -> Option<Cow<'static, str>> {
        let path = into_path.into_field_path();

        let error = if let Some(errors) = &self.errors {
            match path {
                ValidatorPathType::Single(item) => get_error_recursively(errors, &[item]),
                ValidatorPathType::Borrowed(path) => get_error_recursively(errors, path),
            }
        } else {
            None
        };

        if let Some(message) = error
            .and_then(|errors| errors.first())
            .and_then(|e| e.message.as_ref())
        {
            return Some(message.clone());
        }

        error.and_then(|errors| errors.first()).map(|error| {
            if let Some(get_t) = &self.get_t {
                get_t(error)
            } else {
                error.message.clone().unwrap_or_else(|| error.code.clone())
            }
        })
    }

    fn has_errors(&self) -> bool {
        self.errors.is_some()
    }

    fn error_count(&self) -> usize {
        self.errors.as_ref().map_or(0, |errors| errors.0.len())
    }

    fn get_errors(&self) -> Option<&Self::Errors> {
        self.errors.as_ref()
    }
}

impl<'a> IntoFieldPath<ValidatorPathType<'a>> for ValidatorPathType<'a> {
    fn into_field_path(self) -> ValidatorPathType<'a> {
        self
    }
}

impl<'a> IntoFieldPath<ValidatorPathType<'a>> for &'a [PathItem<'a>] {
    fn into_field_path(self) -> ValidatorPathType<'a> {
        ValidatorPathType::Borrowed(self)
    }
}

impl<'a> IntoFieldPath<ValidatorPathType<'a>> for &'a str {
    fn into_field_path(self) -> ValidatorPathType<'a> {
        ValidatorPathType::Single(PathItem::Field(Cow::Borrowed(self)))
    }
}
