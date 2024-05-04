use crate::EguiValidationErrors;
use std::borrow::Borrow;
use std::collections::BTreeMap;

pub struct GardeReport(BTreeMap<String, garde::Error>);

impl GardeReport {
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

impl EguiValidationErrors for GardeReport {
    type Check = String;

    fn get_field_error<B: Eq + Ord + ?Sized>(&self, field: &B) -> Option<String>
    where
        Self::Check: Borrow<B>,
    {
        self.0.get(field).map(|e| e.to_string())
    }

    fn has_errors(&self) -> bool {
        !self.0.is_empty()
    }

    fn error_count(&self) -> usize {
        self.0.len()
    }
}
