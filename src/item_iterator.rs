use crate::item::{Item, ItemResponse};
use crate::state::DragDetectionState;
use crate::{DragDropUi, ItemState};
use egui::{Id, Layout, Rect, Ui, Vec2};

pub struct ItemIterator<'a> {
    state: &'a mut DragDropUi,
    dragged_item_rect: Option<Rect>,
    hovering_item: Option<Id>,
    layout: Layout,
    set_next_item_as_hovering_above: bool,
    direction_vec: Vec2,
    pub(crate) hovering_last_item: bool,
    pub(crate) last_item: Option<(usize, Id)>,

    pub(crate) mark_next_as_closest_item: Option<f32>,

    pub(crate) is_after_dragged_item: bool,
    pub(crate) is_after_hovered_item: bool,
    pub(crate) hovering_over_any_handle: bool,
    pub(crate) source_item: Option<(usize, Id)>,

    pub(crate) closest_item: Option<(f32, Option<(usize, Id)>)>,
}

impl<'a> ItemIterator<'a> {
    pub fn new(state: &'a mut DragDropUi, dragged_item_rect: Option<Rect>, layout: Layout) -> Self {
        let (hovering_item) = match state.detection_state {
            DragDetectionState::Dragging {
                closest_item: item, ..
            } => Some(item),
            _ => None,
        };

        let hovering_last_item = match state.detection_state {
            DragDetectionState::Dragging {
                hovering_last_item, ..
            } => hovering_last_item,
            _ => false,
        };

        Self {
            state,
            dragged_item_rect,
            direction_vec: if layout.is_horizontal() {
                Vec2::X
            } else {
                Vec2::Y
            },
            layout,
            set_next_item_as_hovering_above: false,
            closest_item: None,
            hovering_item,
            mark_next_as_closest_item: None,
            hovering_last_item,
            last_item: None,

            is_after_dragged_item: false,
            is_after_hovered_item: false,
            hovering_over_any_handle: false,
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

        if let Some(distance) = self.mark_next_as_closest_item {
            self.mark_next_as_closest_item = None;
            self.closest_item = Some((distance, Some((idx, id))));
        }

        if is_dragged_item {
            self.is_after_dragged_item = true;
        }

        if let Some(hovering_item) = self.hovering_item {
            if hovering_item == id {
                self.is_after_hovered_item = true;
            }
        }

        if !self.hovering_last_item {
            self.add_space_and_check_closest(ui, id, idx);
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

        if self.hovering_last_item {
            self.add_space_and_check_closest(ui, id, idx);
        }

        if let Some(dragged_item_rect) = self.dragged_item_rect {
            if !self.layout.main_wrap {
                let distance = dragged_item_rect.center().distance(rect.center());
                self.check_closest_item(distance, Some((idx, id)), self.is_after_hovered_item);
            } else {
                if rect.contains(dragged_item_rect.center()) {
                    if self.is_after_hovered_item {
                        self.mark_next_as_closest_item = Some(0.0);
                    } else {
                        self.closest_item = Some((0.0, Some((idx, id))));
                    }
                }
            }
        }

        if self.state.detection_state.is_dragging_item(id) {
            self.source_item = Some((idx, id));
        }

        self.last_item = Some((idx, id));
    }

    fn add_space_and_check_closest(&mut self, ui: &mut Ui, id: Id, idx: usize) {
        if let Some(hovering_item) = self.hovering_item {
            if hovering_item == id {
                // TODO unwrap
                let (id, rect) = ui.allocate_space(self.dragged_item_rect.unwrap().size());

                if let Some(dragged_item_rect) = self.dragged_item_rect {
                    let distance = dragged_item_rect.center().distance(rect.center());
                    self.check_closest_item(distance, None, false);
                }
            }
        }
    }

    fn check_closest_item(&mut self, distance: f32, item: Option<(usize, Id)>, mark_next: bool) {
        if self.closest_item.is_none() || distance < self.closest_item.unwrap().0 {
            if !mark_next {
                self.closest_item = Some((distance, item));
            } else {
                self.mark_next_as_closest_item = Some(distance);
            }
        }
    }
}
