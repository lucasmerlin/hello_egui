use std::borrow::Cow;

pub trait EguiValidationErrors {
    type Check<'a>: Copy;
    fn get_field_error(&self, field: Self::Check<'_>) -> Option<Cow<'static, str>>;

    fn has_errors(&self) -> bool;
    fn error_count(&self) -> usize;
}
