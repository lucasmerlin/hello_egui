use crate::EguiValidationErrors;
use std::borrow::Cow;

use std::hash::Hash;
use validator::{Validate, ValidationError, ValidationErrors, ValidationErrorsKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PathItem {
    Field(Cow<'static, str>),
    Indexed(usize),
}

impl From<usize> for PathItem {
    fn from(value: usize) -> Self {
        PathItem::Indexed(value)
    }
}

impl From<String> for PathItem {
    fn from(value: String) -> Self {
        PathItem::Field(Cow::Owned(value))
    }
}

impl From<&'static str> for PathItem {
    fn from(value: &'static str) -> Self {
        PathItem::Field(Cow::Borrowed(value))
    }
}

#[macro_export]
macro_rules! field_path {
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

pub struct ValidatorReport {
    get_t: Option<GetTranslationFn>,
    errors: Option<ValidationErrors>,
}

impl ValidatorReport {
    pub fn new(result: Result<(), ValidationErrors>) -> Self {
        ValidatorReport {
            errors: result.err(),
            get_t: None,
        }
    }

    pub fn validate<T: Validate>(value: T) -> Self {
        let result = value.validate();
        Self::new(result)
    }

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

impl EguiValidationErrors for ValidatorReport {
    type Check<'a> = &'a [PathItem];

    fn get_field_error(&self, path: Self::Check<'_>) -> Option<Cow<'static, str>> {
        let error = if let Some(errors) = &self.errors {
            get_error_recursively(errors, path)
        } else {
            None
        };

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
}
