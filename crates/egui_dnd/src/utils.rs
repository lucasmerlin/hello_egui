/// Move an item in a slice according to the drag and drop logic.
///
/// Rotates the section of the slice between `source_idx` and `target_idx` such that the item
/// previously at `source_idx` ends up at `target_idx - 1` if `target_idx > source_idx`, and
/// at `target_idx` otherwise. This matches the expected behavior when grabbing the item in
/// the UI and moving it to another position.
///
/// # Example
///
/// ```rust
/// use egui_dnd::utils::shift_vec;
///
/// let mut v = vec![1, 2, 3, 4];
/// shift_vec(1, 1, &mut v);
/// assert_eq!(v, [1, 2, 3, 4]);
/// shift_vec(0, 2, &mut v);
/// assert_eq!(v, [2, 1, 3, 4]);
/// shift_vec(2, 0, &mut v);
/// assert_eq!(v, [3, 2, 1, 4]);
/// ```
///
/// # Panics
/// Panics if `source_idx >= len()` or `target_idx > len()`
/// ```rust,should_panic
/// use egui_dnd::utils::shift_vec;
///
/// let mut v = vec![1];
/// shift_vec(0, 2, &mut v);
/// ```
pub fn shift_vec<T>(source_idx: usize, target_idx: usize, vec: &mut [T]) {
    if let Some(slice) = vec.get_mut(source_idx..target_idx) {
        slice.rotate_left(1.min(slice.len()));
    } else if let Some(slice) = vec.get_mut(target_idx..=source_idx) {
        slice.rotate_right(1.min(slice.len()));
    } else {
        panic!(
            "Failed to move item from index {} to index {}. Slice has {} elements",
            source_idx,
            target_idx,
            vec.len()
        );
    }
}
