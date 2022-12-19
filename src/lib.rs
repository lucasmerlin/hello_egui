pub use state::{DragDropItem, DragDropUi, Handle};

mod state;

pub mod utils {
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
