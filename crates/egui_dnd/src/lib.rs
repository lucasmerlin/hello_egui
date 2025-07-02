#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use egui::{Id, Ui};
pub use state::{DragDropConfig, DragDropItem, DragDropResponse, DragUpdate, Handle};

pub use crate::item_iterator::ItemIterator;
use crate::state::DragDropUi;
use std::hash::Hash;

mod item;
mod item_iterator;
mod state;
/// Helper functions to support the drag and drop functionality
pub mod utils;

/// Helper struct for ease of use.
pub struct Dnd<'a> {
    id: Id,
    ui: &'a mut Ui,
    drag_drop_ui: DragDropUi,
}

/// Main entry point for the drag and drop functionality.
/// Loads and saves it's state from egui memory.
/// Use either [`Dnd::show`] or [`Dnd::show_vec`] to display the drag and drop UI.
/// You can use [`Dnd::with_mouse_config`] or [`Dnd::with_touch_config`] to configure the drag detection.
/// Example usage:
/// ```rust no_run
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
pub fn dnd(ui: &mut Ui, id_source: impl Hash) -> Dnd<'_> {
    let id = Id::new(id_source).with("dnd");
    let mut dnd_ui: DragDropUi =
        ui.data_mut(|data| (*data.get_temp_mut_or_default::<DragDropUi>(id)).clone());

    dnd_ui.return_animation_time = ui.style().animation_time;
    dnd_ui.swap_animation_time = ui.style().animation_time;

    Dnd {
        id,
        ui,
        drag_drop_ui: dnd_ui,
    }
}

impl<'a> Dnd<'a> {
    /// Initialize the drag and drop UI. Same as [dnd].
    pub fn new(ui: &'a mut Ui, id_source: impl Hash) -> Self {
        dnd(ui, id_source)
    }

    /// Sets the config used when dragging with the mouse or when no touch config is set
    #[must_use]
    pub fn with_mouse_config(mut self, config: DragDropConfig) -> Self {
        self.drag_drop_ui = self.drag_drop_ui.with_mouse_config(config);
        self
    }

    /// Sets the config used when dragging with touch
    /// If None, the mouse config is used instead
    /// Use [`DragDropConfig::touch`] or [`DragDropConfig::touch_scroll`] to get a config optimized for touch
    /// The default is [`DragDropConfig::touch`]
    /// For dragging in a `ScrollArea`, use [`DragDropConfig::touch_scroll`]
    #[must_use]
    pub fn with_touch_config(mut self, config: Option<DragDropConfig>) -> Self {
        self.drag_drop_ui = self.drag_drop_ui.with_touch_config(config);
        self
    }

    /// Sets the animation time for the return animation (after dropping an item)
    /// The default is the same as the egui animation time
    /// If you want to disable the animation, set it to 0
    #[must_use]
    pub fn with_return_animation_time(mut self, animation_time: f32) -> Self {
        self.drag_drop_ui.return_animation_time = animation_time;
        self
    }

    /// Sets the animation time for the swap animation (when dragging an item over another item)
    /// The default is the same as the egui animation time
    /// If you want to disable the animation, set it to 0
    #[must_use]
    pub fn with_swap_animation_time(mut self, animation_time: f32) -> Self {
        self.drag_drop_ui.swap_animation_time = animation_time;
        self
    }

    /// Sets the animation time for all animations
    /// The default is the same as the egui animation time
    /// If you want to disable all animations, set it to 0
    #[must_use]
    pub fn with_animation_time(mut self, animation_time: f32) -> Self {
        self.drag_drop_ui.return_animation_time = animation_time;
        self.drag_drop_ui.swap_animation_time = animation_time;
        self
    }

    /// Display the drag and drop UI.
    /// `items` should be an iterator over items that should be sortable.
    /// Each item needs to implement [`DragDropItem`]. This is automatically implement for every type that implements [Hash].
    ///
    /// It can also be implemented manually. **Each item needs to have a unique id.**
    /// If you need to allow duplicate items in your list and cannot add a id field for some reason,
    /// you can use the index as a id, but there are some limitations. Check the [index_as_id](https://github.com/lucasmerlin/hello_egui/blob/main/crates/egui_dnd/examples/index_as_id.rs) example.
    ///
    /// The items won't be updated automatically, but you can use [`Dnd::show_vec`] or [`DragDropResponse::update_vec`] to do so.
    /// If your items aren't in a vec, you have to update the order yourself.
    ///
    /// `item_ui` is called for each item. Display your item there.
    /// `item_ui` gets a [Handle] that can be used to display the drag handle.
    /// Only the handle can be used to drag the item. If you want the whole item to be draggable, put everything in the handle.
    pub fn show<T: DragDropItem>(
        self,
        items: impl Iterator<Item = T>,
        mut item_ui: impl FnMut(&mut Ui, T, Handle<'_>, ItemState),
    ) -> DragDropResponse {
        self.show_with_inner(|_id, ui, drag_drop_ui| {
            drag_drop_ui.ui(ui, |ui, iter| {
                items.enumerate().for_each(|(i, item)| {
                    iter.next(ui, item.id(), i, true, |ui, item_handle| {
                        item_handle.ui(ui, |ui, handle, state| item_ui(ui, item, handle, state))
                    });
                });
            })
        })
    }

    /// Same as [`Dnd::show`], but with a fixed size for each item.
    /// This allows items to be placed in a `horizontal_wrapped` ui.
    /// For more info, look at the [horizontal example](https://github.com/lucasmerlin/hello_egui/blob/main/crates/egui_dnd/examples/horizontal.rs).
    /// If you need even more control over the size, use [`Dnd::show_custom`] instead, where you
    /// can individually size each item. See the `sort_words` example for an example.
    pub fn show_sized<T: DragDropItem>(
        self,
        items: impl Iterator<Item = T>,
        size: egui::Vec2,
        mut item_ui: impl FnMut(&mut Ui, T, Handle<'_>, ItemState),
    ) -> DragDropResponse {
        self.show_with_inner(|_id, ui, drag_drop_ui| {
            drag_drop_ui.ui(ui, |ui, iter| {
                items.enumerate().for_each(|(i, item)| {
                    iter.next(ui, item.id(), i, true, |ui, item_handle| {
                        item_handle.ui_sized(ui, size, |ui, handle, state| {
                            item_ui(ui, item, handle, state);
                        })
                    });
                });
            })
        })
    }

    /// Same as [`Dnd::show`], but automatically sorts the items.
    pub fn show_vec<T: Hash>(
        self,
        items: &mut [T],
        item_ui: impl FnMut(&mut Ui, &mut T, Handle<'_>, ItemState),
    ) -> DragDropResponse {
        let response = self.show(items.iter_mut(), item_ui);
        response.update_vec(items);
        response
    }

    /// Same as [`Dnd::show_sized`], but automatically sorts the items.
    pub fn show_vec_sized<T: Hash>(
        self,
        items: &mut [T],
        size: egui::Vec2,
        item_ui: impl FnMut(&mut Ui, &mut T, Handle<'_>, ItemState),
    ) -> DragDropResponse {
        let response = self.show_sized(items.iter_mut(), size, item_ui);
        response.update_vec(items);
        response
    }

    /// This will allow for very flexible UI. You can use it to e.g. render outlines around items
    /// or render items in complex layouts. This is **experimental**.
    pub fn show_custom(self, f: impl FnOnce(&mut Ui, &mut ItemIterator<'_>)) -> DragDropResponse {
        self.show_with_inner(|_id, ui, drag_drop_ui| drag_drop_ui.ui(ui, f))
    }

    /// Same as [`Dnd::show_custom`], but automatically sorts the items.
    pub fn show_custom_vec<T: Hash>(
        self,
        items: &mut [T],
        f: impl FnOnce(&mut Ui, &mut [T], &mut ItemIterator<'_>),
    ) -> DragDropResponse {
        let response = self.show_custom(|ui, iter| f(ui, items, iter));
        response.update_vec(items);
        response
    }

    fn show_with_inner(
        self,
        inner_fn: impl FnOnce(Id, &mut Ui, &mut DragDropUi) -> DragDropResponse,
    ) -> DragDropResponse {
        let Dnd {
            id,
            ui,
            mut drag_drop_ui,
        } = self;

        let response = inner_fn(id, ui, &mut drag_drop_ui);

        ui.ctx().data_mut(|data| data.insert_temp(id, drag_drop_ui));

        response
    }
}

/// State of the current item.
pub struct ItemState {
    /// True if the item is currently being dragged.
    pub dragged: bool,
    /// Index of the item in the list.
    /// Note that when you sort the source list while the drag is still ongoing (default behaviour
    /// of [`Dnd::show_vec`]), this index will updated while the item is being dragged.
    /// If you sort once after the item is dropped, the index will be stable during the drag.
    pub index: usize,
}
