#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
pub use state::{DragDropItem, DragDropUi, Handle};

mod state;

/// Helper functions to support the drag and drop functionality
pub mod utils {
    /// Move an element from one index to another in a vector, shifting the other elements as
    /// needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use egui_dnd::utils::shift_vec;
    ///
    /// let mut v = vec![1, 2, 3, 4];
    /// shift_vec(0, 2, &mut v);
    /// assert_eq!(v, [2, 1, 3, 4])
    /// ```
    ///
    /// # Panics
    /// Panics if `source_idx` >= len() or `target_idx` > len()
    /// ```rust,should_panic
    /// use egui_dnd::utils::shift_vec;
    ///
    /// let mut v = vec![1];
    /// shift_vec(0, 2, &mut v);
    /// ```
    pub fn shift_vec<T>(source_idx: usize, target_idx: usize, vec: &mut Vec<T>) {
        let target_idx = if source_idx >= target_idx {
            target_idx
        } else {
            target_idx - 1
        };

        let item = vec.remove(source_idx);
        vec.insert(target_idx, item);
    }
}
