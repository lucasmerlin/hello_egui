use crate::item::{Item, ItemResponse};
use crate::state::DragDetectionState;
use crate::{DragDropUi, ItemState};
use egui::{Id, Layout, Pos2, Rect, Ui, UiBuilder, Vec2};

/// Calculates some information that is later used to detect in which index the dragged item should be placed.
/// [`ItemIterator::next`] should be called for each item in the list.
pub struct ItemIterator<'a> {
    state: &'a mut DragDropUi,
    dragged_item_rect: Option<Rect>,
    hovering_item: Option<(Id, Pos2)>,
    layout: Layout,
    set_next_item_as_hovering_above: bool,
    pub(crate) hovering_last_item: bool,
    pub(crate) last_item: Option<(usize, Id, Pos2)>,

    pub(crate) mark_next_as_closest_item: Option<(f32, Pos2)>,

    pub(crate) is_after_dragged_item: bool,
    pub(crate) is_after_hovered_item: bool,
    pub(crate) hovering_over_any_handle: bool,
    pub(crate) source_item: Option<(usize, Id)>,

    #[allow(clippy::type_complexity)]
    pub(crate) closest_item: Option<(f32, Option<(usize, Id, Pos2)>)>,
}

impl<'a> ItemIterator<'a> {
    pub(crate) fn new(
        state: &'a mut DragDropUi,
        dragged_item_rect: Option<Rect>,
        layout: Layout,
    ) -> Self {
        let hovering_item = match state.detection_state {
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

    /// Draw a dnd item. This should be called for each item in the list.
    ///
    /// If `add_surrounding_space_automatically` is false, you need to call
    /// [`ItemIterator::space_before`] and [`ItemIterator::space_after`] manually.
    /// This is useful, e.g. to add a divider between items. Check the custom ui example.
    pub fn next(
        &mut self,
        ui: &mut Ui,
        id: Id,
        idx: usize,
        add_surrounding_space_automatically: bool,
        content: impl FnOnce(&mut Ui, Item) -> ItemResponse,
    ) {
        let is_dragged_item = self.state.detection_state.is_dragging_item(id);

        if let Some((distance, pos)) = self.mark_next_as_closest_item {
            self.mark_next_as_closest_item = None;
            self.closest_item = Some((distance, Some((idx, id, pos))));
        }

        if is_dragged_item {
            self.is_after_dragged_item = true;
        }

        if let Some((hovering_id, _pos)) = self.hovering_item {
            if hovering_id == id {
                self.is_after_hovered_item = true;
            }
        }

        if add_surrounding_space_automatically {
            self.space_before(ui, id, |_ui, _space| {});
        }

        let dragging = self.state.detection_state.is_dragging();

        let item = Item::new(
            id,
            ItemState {
                dragged: is_dragged_item,
                index: idx,
            },
            self.state,
            &mut self.hovering_over_any_handle,
        );
        let rect = if is_dragged_item {
            if let Some((_id, pos)) = self.hovering_item {
                let mut child =
                    ui.new_child(UiBuilder::new().max_rect(ui.available_rect_before_wrap()));
                let start = ui.next_widget_position();
                let rect = child
                    .allocate_new_ui(
                        UiBuilder::new().max_rect(Rect::from_min_size(pos, child.available_size())),
                        |ui| content(ui, item),
                    )
                    .inner
                    .0;
                Rect::from_min_size(start, rect.size())
            } else {
                content(ui, item).0
            }
        } else {
            content(ui, item).0
        };

        if dragging != self.state.detection_state.is_dragging() {
            self.set_next_item_as_hovering_above = true;
        }

        if add_surrounding_space_automatically {
            self.space_after(ui, id, |_ui, _space| {});
        }

        if let Some(dragged_item_rect) = self.dragged_item_rect {
            if self.layout.main_wrap {
                if rect.contains(dragged_item_rect.center()) {
                    if self.is_after_hovered_item {
                        self.mark_next_as_closest_item = Some((0.0, rect.min));
                    } else {
                        self.closest_item = Some((0.0, Some((idx, id, rect.min))));
                    }
                }
            } else {
                let (distance, mark_next) = self.get_distance(dragged_item_rect, rect);
                self.check_closest_item(distance, rect.min, Some((idx, id)), mark_next);
            }
        }

        if self.state.detection_state.is_dragging_item(id) {
            self.source_item = Some((idx, id));
        }

        self.last_item = Some((idx, id, rect.min));
    }

    fn get_distance(&mut self, dragged_item_rect: Rect, rect: Rect) -> (f32, bool) {
        let size_difference = dragged_item_rect.size() - rect.size();
        let (distance, mark_next) = if self.layout.is_horizontal() {
            let distance = dragged_item_rect.center().x - rect.center().x;
            let mark_next = rect.center().x < dragged_item_rect.center().x;
            (distance, mark_next)
        } else {
            let distance = dragged_item_rect.center().y - rect.center().y;
            let mark_next = if size_difference.y.abs() > 0.0 {
                rect.center().y < dragged_item_rect.center().y
            } else {
                self.is_after_hovered_item
            };
            (distance, mark_next)
        };
        let distance = distance.abs();
        (distance, mark_next)
    }

    /// Add some custom ui before the item.
    pub fn space_before(&mut self, ui: &mut Ui, id: Id, content: impl FnOnce(&mut Ui, Vec2)) {
        if !self.hovering_last_item {
            self.add_space_and_check_closest(ui, id, content);
        }
    }

    /// Add some custom ui after the item.
    pub fn space_after(&mut self, ui: &mut Ui, id: Id, content: impl FnOnce(&mut Ui, Vec2)) {
        if self.hovering_last_item {
            self.add_space_and_check_closest(ui, id, content);
        }
    }

    fn add_space_and_check_closest(
        &mut self,
        ui: &mut Ui,
        id: Id,
        content: impl FnOnce(&mut Ui, Vec2),
    ) {
        if let Some((hovering_id, _pos)) = self.hovering_item {
            if hovering_id == id {
                if let Some(dragged_item_rect) = self.dragged_item_rect {
                    let rect = ui
                        .allocate_ui(dragged_item_rect.size(), |ui| {
                            ui.set_min_size(dragged_item_rect.size());
                            content(ui, dragged_item_rect.size());
                        })
                        .response
                        .rect;
                    let (distance, _mark_next) = self.get_distance(dragged_item_rect, rect);
                    self.check_closest_item(distance, rect.min, None, false);
                }
            }
        }
    }

    fn check_closest_item(
        &mut self,
        distance: f32,
        pos: Pos2,
        item: Option<(usize, Id)>,
        mark_next: bool,
    ) {
        if self.closest_item.is_none() || distance < self.closest_item.unwrap().0 {
            if mark_next {
                self.mark_next_as_closest_item = Some((distance, pos));
            } else {
                self.closest_item = Some((distance, item.map(|(idx, id)| (idx, id, pos))));
            }
        }
    }
}
