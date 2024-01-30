use crate::state::DragDropUi;
use egui::{Id, Rect, Ui};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct DragDropMetaState {
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub(crate) struct CurrentlyDraggedItem {
    pub id: Id,
    pub source_list: Id,
    pub rect: Rect,
}

/// A context that can hold multiple drag and drop lists.
#[derive(Debug, Clone, Default)]
pub struct DndContext {
    uis: HashMap<Id, DragDropMetaState>,
    currently_dragged_item: Option<CurrentlyDraggedItem>,
}

impl DndContext {
    pub(crate) fn should_i_evaluate_a_dragged_item_from_another_list(
        &self,
        id: Id,
    ) -> Option<CurrentlyDraggedItem> {
        if let Some(currently_dragged_item) = &self.currently_dragged_item {
            if currently_dragged_item.source_list != id {
                return Some(currently_dragged_item.clone());
            }
        }
        None
    }

    pub(crate) fn set_meta_state(
        &mut self,
        list_id: Id,
        meta: DragDropMetaState,
        dragged_item: Option<CurrentlyDraggedItem>,
    ) {
        self.uis.insert(list_id, meta);
        if let Some(currently_dragged_item) = dragged_item {
            self.currently_dragged_item = Some(currently_dragged_item);
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, content: impl FnOnce(&mut Ui, &mut Self)) {
        let last_dragged_item = self.currently_dragged_item.take();
        let last_uis = self.uis.clone();

        content(ui, self);
    }
}
