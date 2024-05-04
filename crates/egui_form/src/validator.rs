use crate::EguiValidationErrors;
use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
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

type GetTranslationFn = Box<dyn Fn(&ValidationError) -> Cow<str>>;

pub struct ValidatorReport {
    get_t: Option<GetTranslationFn>,
    errors: HashMap<Vec<PathItem>, Vec<ValidationError>>,
}

impl ValidatorReport {
    pub fn new(result: Result<(), ValidationErrors>) -> Self {
        let mut map = HashMap::default();
        if let Err(errors) = result {
            build_errors(errors, &[], &mut |path, error| {
                map.insert(path.to_vec(), error);
            });
        }

        ValidatorReport {
            errors: map,
            get_t: None,
        }
    }

    pub fn validate<T: Validate>(value: T) -> Self {
        let result = value.validate();
        Self::new(result)
    }

    pub fn with_translation<F: Fn(&ValidationError) -> Cow<str> + 'static>(
        mut self,
        get_t: F,
    ) -> Self {
        self.get_t = Some(Box::new(get_t));
        self
    }
}

fn build_errors(
    errors: validator::ValidationErrors,
    path: &[PathItem],
    callback: &mut impl FnMut(Vec<PathItem>, Vec<ValidationError>),
) {
    for (field, error) in errors.into_errors() {
        match error {
            ValidationErrorsKind::Struct(errors) => {
                let mut path = path.to_vec();
                path.push(PathItem::Field(Cow::Borrowed(field)));
                build_errors(*errors, &path, callback);
            }
            ValidationErrorsKind::List(list) => {
                let mut path = path.to_vec();
                path.push(PathItem::Field(Cow::Borrowed(field)));
                path.push(PathItem::Indexed(0));
                for (i, error) in list.into_iter() {
                    *path.last_mut().unwrap() = PathItem::Indexed(i);
                    build_errors(*error, &path, callback);
                }
            }
            ValidationErrorsKind::Field(errors) => {
                let mut path = path.to_vec();
                path.push(PathItem::Field(Cow::Borrowed(field)));
                callback(path, errors);
            }
        }
    }
}

impl EguiValidationErrors for ValidatorReport {
    type Check = Vec<PathItem>;

    fn get_field_error<B: Hash + Eq + ?Sized>(&self, field: &B) -> Option<String>
    where
        Self::Check: Borrow<B>,
    {
        self.errors
            .get(field)
            .and_then(|errors| errors.first())
            .map(|error| {
                if let Some(get_t) = &self.get_t {
                    get_t(error).into_owned()
                } else {
                    error
                        .message
                        .clone()
                        .unwrap_or_else(|| error.code.clone())
                        .to_string()
                }
            })
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn error_count(&self) -> usize {
        self.errors.values().map(|v| v.len()).sum()
    }
}
