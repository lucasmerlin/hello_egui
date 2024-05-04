use std::borrow::Borrow;
use std::hash::Hash;

pub trait EguiValidationErrors {
    type Check: Ord + Eq + Hash;
    fn get_field_error<B: Hash + Eq + Ord + ?Sized>(&self, field: &B) -> Option<String>
    where
        Self::Check: Borrow<B>;

    fn has_errors(&self) -> bool;
    fn error_count(&self) -> usize;
}
