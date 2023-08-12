use crate::item::{Item, ItemResponse};
use crate::state::{DragDetectionState, DragPhase};
use crate::{DragDropUi, ItemState};
use egui::{Id, Layout, Rect, Ui, Vec2};

pub struct ItemIterator<'a> {
    state: &'a mut DragDropUi,
    dragged_item_rect: Option<Rect>,
    layout: Layout,
    dragged_item: Option<(Option<Id>, Option<Id>)>,
    set_next_item_as_hovering_above: bool,

    pub(crate) is_after_dragged_item: bool,
    pub(crate) hovering_over_any_handle: bool,
    pub(crate) before_item: Option<(usize, Id)>,
    pub(crate) after_item: Option<(usize, Id)>,
    pub(crate) source_item: Option<(usize, Id)>,
}

impl<'a> ItemIterator<'a> {
    pub fn new(state: &'a mut DragDropUi, dragged_item_rect: Option<Rect>, layout: Layout) -> Self {
        let dragged_item = match state.detection_state {
            DragDetectionState::Dragging {
                hovering_above_item,
                hovering_below_item,
                ..
            } => Some((hovering_above_item, hovering_below_item)),
            _ => None,
        };

        dbg!(dragged_item);

        Self {
            dragged_item: dragged_item,
            state,
            dragged_item_rect,
            layout,
            set_next_item_as_hovering_above: false,

            is_after_dragged_item: false,
            hovering_over_any_handle: false,
            before_item: None,
            after_item: None,
            source_item: None,
        }
    }

    pub fn next<T>(
        &mut self,
        ui: &mut Ui,
        id: Id,
        item: T,
        idx: usize,
        content: impl FnOnce(&mut Ui, Item<T>) -> ItemResponse,
    ) {
        let is_dragged_item = self.state.detection_state.is_dragging_item(id);
        if is_dragged_item {
            self.is_after_dragged_item = true;
        }

        // if self.set_next_item_as_hovering_above {
        //     self.set_next_item_as_hovering_above = false;
        //     if let DragDetectionState::Dragging {
        //         hovering_below_item,
        //         ..
        //     } = &mut self.state.detection_state
        //     {
        //         *hovering_below_item = Some(id);
        //     }
        // }

        if let Some((hovering_above_item, hovering_below_item)) = self.dragged_item {
            if hovering_above_item == Some(id) {
                // TODO unwrap
                ui.allocate_space(self.dragged_item_rect.unwrap().size());
            }
            if hovering_below_item == Some(id) && hovering_above_item.is_none() {
                // TODO unwrap
                ui.allocate_space(self.dragged_item_rect.unwrap().size());
            }
        }

        let dragging = self.state.detection_state.is_dragging();

        let ItemResponse { rect, .. } = content(
            ui,
            Item::new(
                id,
                item,
                ItemState {
                    dragged: is_dragged_item,
                    index: idx,
                },
                self.state,
                &mut self.hovering_over_any_handle,
            ),
        );

        if dragging != self.state.detection_state.is_dragging() {
            self.set_next_item_as_hovering_above = true;
        }

        let additional_margin = if self.is_after_dragged_item {
            rect.size()
        } else {
            Vec2::ZERO
        };

        if let Some(dragged_item_rect) = self.dragged_item_rect {
            if self.layout.is_horizontal() {
                if !self.layout.main_wrap
                    || (dragged_item_rect.center().y < rect.max.y
                        && dragged_item_rect.center().y > rect.min.y)
                {
                    if dragged_item_rect.center().x < rect.max.x - additional_margin.x
                        && self.before_item.is_none()
                    {
                        self.before_item = Some((idx, id));
                    }
                    if dragged_item_rect.center().x > rect.min.x {
                        self.after_item = Some((idx, id));
                    }
                }
            } else {
                // TODO: Use .top and .bottom here for more optimistic switching
                if dragged_item_rect.center().y < rect.max.y - additional_margin.y
                    && self.before_item.is_none()
                {
                    self.before_item = Some((idx, id));
                }
                if dragged_item_rect.center().y > rect.min.y {
                    self.after_item = Some((idx, id));
                }
            }
        }

        if self.state.detection_state.is_dragging_item(id) {
            self.source_item = Some((idx, id));
        }
    }
}
