#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use crate::state::DragDropResponse;
use egui::Id;
pub use state::{DragDropItem, DragDropUi, Handle};
use std::borrow::BorrowMut;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::os::unix::raw::time_t;

mod state;

/// Helper functions to support the drag and drop functionality
pub mod utils {
    /// Move an item in a slice according to the drag and drop logic.
    ///
    /// Rotates the section of the slice between `source_idx` and `target_idx` such that the item
    /// previously at `source_idx` ends up at `target_idx - 1` if `target_idx > source_idx`, and
    /// at `target_idx` otherwhise. This matches the expected behavior when grabbing the item in
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
}

pub struct Dnd<'a> {
    id: Id,
    ui: &'a mut egui::Ui,
    drag_drop_ui: DragDropUi,
}

impl<'a> Deref for Dnd<'a> {
    type Target = DragDropUi;

    fn deref(&self) -> &Self::Target {
        &self.drag_drop_ui
    }
}

impl<'a> DerefMut for Dnd<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.drag_drop_ui
    }
}
/// Main entry point for the drag and drop functionality.
/// Loads and saves it's state from egui memory.
/// Use either [Dnd::show] or [Dnd::show_vec] to display the drag and drop UI.
/// You can use [DragDropUi::with_mouse_config] or [DragDropUi::with_touch_config] to configure the drag detection.
/// Example usage:
/// ```rust
/// use std::hash::Hash;
/// use eframe::egui;
/// use egui::CentralPanel;
/// use egui_dnd::dnd;
///
/// pub fn main() -> eframe::Result<()> {
///     let mut items = vec!["alfred", "bernhard", "christian"];
///
///     eframe::run_simple_native("DnD Simple Example", Default::default(), move |ctx, _frame| {
///         CentralPanel::default().show(ctx, |ui| {
///
///             dnd(ui, "dnd_example")
///                 .show_vec(&mut items, |ui, item, handle, state| {
///                     handle.ui(ui, |ui| {
///                         ui.label("drag");
///                     });
///                     ui.label(*item);
///                 });
///
///         });
///     })
/// }
///
/// ```
pub fn dnd(ui: &mut egui::Ui, id_source: impl Hash) -> Dnd {
    let id = Id::new(id_source).with("dnd");
    let dnd_ui: DragDropUi =
        ui.data_mut(|data| (*data.get_temp_mut_or_default::<DragDropUi>(id)).clone());

    Dnd {
        id,
        ui,
        drag_drop_ui: dnd_ui,
    }
}

impl<'a> Dnd<'a> {
    pub fn new(ui: &'a mut egui::Ui, id_source: impl Hash) -> Self {
        dnd(ui, id_source)
    }

    pub fn show<T: DragDropItem>(
        self,
        items: impl Iterator<Item = T>,
        mut item_ui: impl FnMut(&mut egui::Ui, T, Handle, ItemState),
    ) -> DragDropResponse {
        let Dnd {
            id,
            ui,
            mut drag_drop_ui,
        } = self;

        let response = drag_drop_ui.ui(ui, items, |item, ui, handle, dragged| {
            item_ui(ui, item, handle, ItemState { dragged });
        });

        ui.ctx().data_mut(|data| data.insert_temp(id, drag_drop_ui));

        response
    }

    //
    pub fn show_vec<T: Hash>(
        self,
        items: &mut Vec<T>,
        mut item_ui: impl FnMut(&mut egui::Ui, &mut T, Handle, ItemState),
    ) -> DragDropResponse {
        let i = &mut items[0];

        i.id();

        let response = self.show(items.iter_mut(), item_ui);
        response.update_vec(items);
        response
    }
}

pub struct ItemState {
    pub dragged: bool,
}
