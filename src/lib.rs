#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::Id;
pub use state::{DragDropConfig, DragDropItem, DragDropResponse, DragDropUi, DragUpdate, Handle};

use std::hash::Hash;

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

/// Helper struct for ease of use.
pub struct Dnd<'a> {
    id: Id,
    ui: &'a mut egui::Ui,
    drag_drop_ui: DragDropUi,
}

/// Main entry point for the drag and drop functionality.
/// Loads and saves it's state from egui memory.
/// Use either [Dnd::show] or [Dnd::show_vec] to display the drag and drop UI.
/// You can use [DragDropUi::with_mouse_config] or [DragDropUi::with_touch_config] to configure the drag detection.
/// Example usage:
/// ```rust;no_run
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
    /// Initialize the drag and drop UI. Same as [dnd].
    pub fn new(ui: &'a mut egui::Ui, id_source: impl Hash) -> Self {
        dnd(ui, id_source)
    }

    /// Sets the config used when dragging with the mouse or when no touch config is set
    pub fn with_mouse_config(mut self, config: DragDropConfig) -> Self {
        self.drag_drop_ui = self.drag_drop_ui.with_mouse_config(config);
        self
    }

    /// Sets the config used when dragging with touch
    /// If None, the mouse config is used instead
    /// Use [DragDropConfig::touch] or [DragDropConfig::touch_scroll] to get a config optimized for touch
    /// The default is [DragDropConfig::touch]
    /// For dragging in a ScrollArea, use [DragDropConfig::touch_scroll]
    pub fn with_touch_config(mut self, config: Option<DragDropConfig>) -> Self {
        self.drag_drop_ui = self.drag_drop_ui.with_touch_config(config);
        self
    }

    /// Display the drag and drop UI.
    /// `items` should be an iterator over items that should be sorted.
    ///
    /// The items won't be sorted automatically, but you can use [Dnd::show_vec] or [DragDropResponse::update_vec] to do so.
    /// If your items aren't in a vec, you have to sort them yourself.
    ///
    /// `item_ui` is called for each item. Display your item there.
    /// `item_ui` gets a [Handle] that can be used to display the drag handle.
    /// Only the handle can be used to drag the item. If you want the whole item to be draggable, put everything in the handle.
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

        let response = drag_drop_ui.ui(ui, items, |item, ui, handle, item_state| {
            item_ui(ui, item, handle, item_state);
        });

        ui.ctx().data_mut(|data| data.insert_temp(id, drag_drop_ui));

        response
    }

    /// Same as [Dnd::show], but automatically sorts the items.
    pub fn show_vec<T: Hash>(
        self,
        items: &mut [T],
        item_ui: impl FnMut(&mut egui::Ui, &mut T, Handle, ItemState),
    ) -> DragDropResponse {
        let response = self.show(items.iter_mut(), item_ui);
        response.update_vec(items);
        response
    }
}

/// State of the current item.
pub struct ItemState {
    /// True if the item is currently being dragged.
    pub dragged: bool,
    /// Index of the item in the list.
    /// Note that when you sort the source list while the drag is still ongoing (default behaviour
    /// of [Dnd::show_vec]), this index will updated while the item is being dragged.
    /// If you sort once after the item is dropped, the index will be stable during the drag.
    pub index: usize,
}
