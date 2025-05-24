#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, SystemTime};

use egui::{CursorIcon, Id, Pos2, Rect, Sense, Ui, Vec2};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, SystemTime};

use crate::item_iterator::ItemIterator;
use crate::utils::shift_vec;

/// Item that can be reordered using drag and drop
pub trait DragDropItem {
    /// Unique id for the item, to allow egui to keep track of its dragged state between frames
    fn id(&self) -> Id;
}

impl<T: std::hash::Hash> DragDropItem for T {
    fn id(&self) -> Id {
        Id::new(self)
    }
}

/// An instruction in what order to update the source list.
/// The item at from should be removed from the list and inserted at to.
/// You can use [`shift_vec`] to do this for a Vec.
#[derive(Debug, Clone)]
pub struct DragUpdate {
    /// Index of the item to move
    pub from: usize,
    /// Where to move the item to
    pub to: usize,
}

/// Response containing state of the drag & drop list and a potential update to the source list.
/// The update can be applied immediately or at latest when [`DragDropResponse::is_drag_finished`] returns true.
#[derive(Debug, Clone)]
pub struct DragDropResponse {
    state: DragDetectionState,
    /// Contains ongoing information about which index is currently being dragged where.
    /// You can use this to consistently update the source list while the drag & drop event is ongoing.
    /// If you only want to update the source list when the drag & drop event has finished, use [`DragDropResponse::final_update`] instead.
    pub update: Option<DragUpdate>,
    finished: bool,
    cancellation_reason: Option<&'static str>,
    has_changed: bool,
}

impl DragDropResponse {
    /// Returns true if we are currently evaluating whether a drag should be started.
    #[must_use]
    pub fn is_evaluating_drag(&self) -> bool {
        self.state.is_evaluating_drag()
    }

    /// Returns true if we are currently dragging an item.
    #[must_use]
    pub fn is_dragging(&self) -> bool {
        self.state.is_dragging()
    }

    /// Returns the id of the item that is currently being dragged.
    #[must_use]
    pub fn dragged_item_id(&self) -> Option<Id> {
        self.state.dragged_item()
    }

    /// Returns true if the drag & drop event has finished and the item has been dropped.
    /// The update should be applied to the source list.
    #[must_use]
    pub fn is_drag_finished(&self) -> bool {
        self.finished
    }

    /// Utility function to update a Vec with the current drag & drop state.
    /// You can use this to consistently update the source list while the drag & drop event is ongoing.
    pub fn update_vec<T>(&self, vec: &mut [T]) {
        if self.has_changed || self.finished {
            if let Some(update) = &self.update {
                shift_vec(update.from, update.to, vec);
            }
        }
    }

    /// Returns the update if the drag & drop event has finished and the item has been dropped.
    /// Useful for the if let syntax.
    #[must_use]
    pub fn final_update(&self) -> Option<DragUpdate> {
        if self.finished {
            self.update.clone()
        } else {
            None
        }
    }

    /// Returns a [Option<&str>] with the reason if a drag & drop event was cancelled.
    #[must_use]
    pub fn cancellation_reason(&self) -> Option<&'static str> {
        self.cancellation_reason
    }
}

/// Holds the data needed to draw the floating item while it is being dragged
/// Deprecated: Use [`crate::dnd`] or [`crate::Dnd::new`] instead
#[derive(Clone, Debug)]
pub struct DragDropUi {
    pub(crate) detection_state: DragDetectionState,
    /// If the mobile config is set, we will use it if we detect a touch event
    touch_config: Option<DragDropConfig>,
    mouse_config: DragDropConfig,
    pub(crate) swap_animation_time: f32,
    pub(crate) return_animation_time: f32,
}

impl Default for DragDropUi {
    fn default() -> Self {
        DragDropUi {
            detection_state: DragDetectionState::None,
            touch_config: Some(DragDropConfig::touch()),
            mouse_config: DragDropConfig::mouse(),
            swap_animation_time: 0.2,
            return_animation_time: 0.2,
        }
    }
}

/// [`Handle::ui`] is used to draw the drag handle
pub struct Handle<'a> {
    id: Id,
    idx: usize,
    state: &'a mut DragDropUi,
    hovering_over_any_handle: &'a mut bool,
    item_pos: Pos2,

    // Configurable options
    sense: Option<Sense>,
    show_drag_cursor_on_hover: bool,
    disable_selectable_labels: bool,
}

#[derive(Debug, Default, Clone)]
pub(crate) enum DragDetectionState {
    #[default]
    None,
    PressedWaitingForDelay {
        pressed_at: SystemTime,
    },
    WaitingForClickThreshold {
        pressed_at: SystemTime,
    },
    CouldBeValidDrag,
    Cancelled(&'static str),
    Dragging {
        id: Id,
        source_idx: usize,
        offset: Vec2,
        dragged_item_size: Vec2,
        closest_item: (Id, Pos2),
        last_pointer_pos: Pos2,
        hovering_last_item: bool,

        // These should only be used for output, as to not cause issues when item indexes change
        hovering_idx: usize,
    },
    TransitioningBackAfterDragFinished {
        id: Id,
        dragged_item_size: Option<Vec2>,
    },
}

impl DragDetectionState {
    fn is_evaluating_drag(&self) -> bool {
        matches!(self, DragDetectionState::WaitingForClickThreshold { .. })
            || matches!(self, DragDetectionState::PressedWaitingForDelay { .. })
            || matches!(self, DragDetectionState::CouldBeValidDrag)
    }

    pub(crate) fn is_dragging(&self) -> bool {
        matches!(self, DragDetectionState::Dragging { .. })
    }

    fn dragged_item(&self) -> Option<Id> {
        match self {
            DragDetectionState::Dragging { id, .. } => Some(*id),
            _ => None,
        }
    }

    pub(crate) fn is_dragging_item(&self, id: Id) -> bool {
        self.dragged_item() == Some(id)
    }

    pub(crate) fn dragged_item_size(&self) -> Option<Vec2> {
        match self {
            DragDetectionState::TransitioningBackAfterDragFinished {
                dragged_item_size: Some(dragged_item_size),
                ..
            }
            | DragDetectionState::Dragging {
                dragged_item_size, ..
            } => Some(*dragged_item_size),
            _ => None,
        }
    }

    pub(crate) fn last_pointer_pos(&self) -> Option<Pos2> {
        match self {
            DragDetectionState::Dragging {
                last_pointer_pos, ..
            } => Some(*last_pointer_pos),
            _ => None,
        }
    }
}

impl<'a> Handle<'a> {
    pub(crate) fn new(
        id: Id,
        idx: usize,
        state: &'a mut DragDropUi,
        hovering_over_any_handle: &'a mut bool,
        item_pos: Pos2,
    ) -> Self {
        Handle {
            id,
            idx,
            state,
            hovering_over_any_handle,
            item_pos,

            sense: None,
            show_drag_cursor_on_hover: true,
            disable_selectable_labels: true,
        }
    }

    /// You can add [`Sense::click`] if you want to listen for clicks on the handle
    /// **Warning**: This will make anything sensing clicks in the handle not draggable
    /// Make sure to not set this if your handle consists of a single button, and directly
    /// query the button for clicks.
    #[must_use]
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = Some(sense);
        self
    }

    /// If `true`, the cursor will change to a grab cursor when hovering over the handle
    /// This is `true` by default
    #[must_use]
    pub fn show_drag_cursor_on_hover(mut self, show: bool) -> Self {
        self.show_drag_cursor_on_hover = show;
        self
    }

    /// By default, selectable labels are disabled in the handle, to not interfere with dragging.
    /// You can use this to re-enable them.
    /// Note that if you disable selectable labels globally, this won't have any effect.
    #[must_use]
    pub fn enable_selectable_labels(mut self) -> Self {
        self.disable_selectable_labels = false;
        self
    }

    /// Draw the drag handle. Use [`Handle::sense`] to add a click sense.
    /// You can also add buttons in the handle, but they won't be interactive if you pass `Sense::click`
    pub fn ui(mut self, ui: &mut Ui, contents: impl FnOnce(&mut Ui)) -> egui::Response {
        let disabled = if self.disable_selectable_labels {
            let interaction = &mut ui.style_mut().interaction;
            let old_values = (
                interaction.selectable_labels,
                interaction.multi_widget_text_select,
            );
            interaction.selectable_labels = false;
            interaction.multi_widget_text_select = false;
            Some(old_values)
        } else {
            None
        };

        let response = ui.push_id(self.id.with("handle_ui"), |ui| contents(ui));

        if let Some((selectable_labels, multi_widget_text_select)) = disabled {
            ui.style_mut().interaction.selectable_labels = selectable_labels;
            ui.style_mut().interaction.multi_widget_text_select = multi_widget_text_select;
        }

        self.handle_response(response.response, ui)
    }

    /// This is useful if you want to sort items in a `horizontal_wrapped`.
    /// This doesn't create a new scope.
    pub fn ui_sized(
        mut self,
        ui: &mut Ui,
        size: Vec2,
        add_contents: impl FnOnce(&mut Ui),
    ) -> egui::Response {
        let response = ui.allocate_ui(size, |ui| {
            if self.disable_selectable_labels {
                ui.style_mut().interaction.selectable_labels = false;
                ui.style_mut().interaction.multi_widget_text_select = false;
            }
            // We somehow have to push a new id here or there will be an id clash at response.interact
            ui.push_id(self.id.with("handle_ui"), add_contents)
        });
        self.handle_response(response.inner.response, ui)
    }

    fn handle_response(&mut self, response: egui::Response, ui: &mut Ui) -> egui::Response {
        let response = if let Some(sense) = self.sense {
            response.interact(sense)
        } else {
            response
        };

        if response.contains_pointer() {
            if self.show_drag_cursor_on_hover {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grab);
            }
            *self.hovering_over_any_handle = true;
        }

        let offset = self.item_pos.to_vec2()
            - ui.ctx()
                .input(|i| i.pointer.hover_pos())
                .unwrap_or_default()
                .to_vec2();

        let drag_distance = ui.input(|i| {
            (i.pointer.hover_pos().unwrap_or_default()
                - i.pointer.press_origin().unwrap_or_default())
            .length()
        });

        let click_threshold = 1.0;
        let is_above_click_threshold = drag_distance > click_threshold;

        if response.contains_pointer()
            && response
                .rect
                .contains(ui.input(|input| input.pointer.press_origin().unwrap_or_default()))
        {
            if let DragDetectionState::WaitingForClickThreshold { pressed_at } =
                self.state.detection_state
            {
                // It should be safe to stop anything else being dragged here
                // This is important so any ScrollArea isn't being dragged while we wait for the click threshold
                ui.ctx().stop_dragging();
                if is_above_click_threshold
                    || pressed_at.elapsed().unwrap_or_default()
                        > self.state.config(ui).click_tolerance_timeout
                {
                    self.state.detection_state = DragDetectionState::CouldBeValidDrag;
                }
            }
        };

        if response.contains_pointer()
            && matches!(
                self.state.detection_state,
                DragDetectionState::CouldBeValidDrag
            )
        {
            self.state.detection_state = DragDetectionState::Dragging {
                id: self.id,
                offset,
                // We set this in the Item
                dragged_item_size: Vec2::default(),
                closest_item: (self.id, self.item_pos),
                source_idx: self.idx,
                hovering_idx: self.idx,
                last_pointer_pos: ui
                    .ctx()
                    .input(|i| i.pointer.hover_pos())
                    .unwrap_or_default(),
                hovering_last_item: false,
            };
            ui.ctx().set_dragged_id(self.id);
        }

        response
    }
}

/// Configuration for drag detection.
#[derive(Debug, Clone)]
pub struct DragDropConfig {
    /// How long does the user have to keep pressing until a drag may begin?
    /// This is useful when dragging and dropping on a touch screen in a scrollable area.
    pub drag_delay: Duration,
    /// How far can the pointer move during the [`DragDropConfig::drag_delay`] before the drag is cancelled?
    pub scroll_tolerance: Option<f32>,
    /// How far does the pointer have to move before a drag starts?
    /// This is useful when the handle is also a button.
    /// If the pointer is released before this threshold, the drag never starts and the button / handle can be clicked.
    /// If you want to detect clicks on the handle itself, [`Handle::sense`] to add a click sense to the handle.
    pub click_tolerance: f32,
    /// If we have been holding longer than this duration, a drag will be started even if the pointer has not moved above [`DragDropConfig::click_tolerance`].
    pub click_tolerance_timeout: Duration,
}

impl Default for DragDropConfig {
    fn default() -> Self {
        Self::mouse()
    }
}

impl DragDropConfig {
    /// Optimized for mouse usage
    #[must_use]
    pub fn mouse() -> Self {
        Self {
            click_tolerance: 1.0,
            drag_delay: Duration::from_millis(0),
            scroll_tolerance: None,
            click_tolerance_timeout: Duration::from_millis(250),
        }
    }

    /// Optimized for touch usage in a fixed size area (no scrolling)
    /// Has a higher click tolerance than [`DragDropConfig::mouse`]
    #[must_use]
    pub fn touch() -> Self {
        Self {
            scroll_tolerance: None,
            click_tolerance: 3.0,
            drag_delay: Duration::from_millis(0),
            click_tolerance_timeout: Duration::from_millis(250),
        }
    }

    /// Optimized for touch usage in a scrollable area
    #[must_use]
    pub fn touch_scroll() -> Self {
        Self {
            scroll_tolerance: Some(6.0),
            click_tolerance: 3.0,
            drag_delay: Duration::from_millis(300),
            click_tolerance_timeout: Duration::from_millis(250),
        }
    }
}

/// [`DragDropUi`] stores the state of the Drag & Drop list.
impl DragDropUi {
    /// Sets the config used when dragging with the mouse or when no touch config is set
    pub fn with_mouse_config(mut self, config: DragDropConfig) -> Self {
        self.mouse_config = config;
        self
    }

    /// Sets the config used when dragging with touch
    /// If None, the mouse config is used instead
    /// Use [`DragDropConfig::touch`] or [`DragDropConfig::touch_scroll`] to get a config optimized for touch
    /// The default is [`DragDropConfig::touch`]
    /// For dragging in a `ScrollArea`, use [`DragDropConfig::touch_scroll`]
    pub fn with_touch_config(mut self, config: Option<DragDropConfig>) -> Self {
        self.touch_config = config;
        self
    }

    fn config(&self, ui: &Ui) -> &DragDropConfig {
        if ui.input(egui::InputState::any_touches) {
            self.touch_config.as_ref().unwrap_or(&self.mouse_config)
        } else {
            &self.mouse_config
        }
    }

    /// Draw the items and handle drag & drop stuff
    #[allow(clippy::too_many_lines)] // TODO: refactor this to reduce the number of lines
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        callback: impl FnOnce(&mut Ui, &mut ItemIterator),
    ) -> DragDropResponse {
        // During the first frame, we check if the pointer is actually over any of the item handles and cancel the drag if it isn't
        let mut first_frame = false;
        let config = self.config(ui).clone();

        ui.input(|i| {
            if i.pointer.any_down() {
                if matches!(self.detection_state, DragDetectionState::None)
                    || matches!(
                        self.detection_state,
                        DragDetectionState::TransitioningBackAfterDragFinished { .. }
                    )
                {
                    first_frame = true;
                    self.detection_state = DragDetectionState::PressedWaitingForDelay {
                        pressed_at: SystemTime::now(),
                    };
                }

                let drag_distance = (i.pointer.hover_pos().unwrap_or_default()
                    - i.pointer.press_origin().unwrap_or_default())
                .length();
                let is_below_scroll_threshold =
                    drag_distance < config.scroll_tolerance.unwrap_or(f32::INFINITY);

                if let DragDetectionState::PressedWaitingForDelay { pressed_at } =
                    self.detection_state
                {
                    if pressed_at.elapsed().unwrap_or_default() >= config.drag_delay {
                        if is_below_scroll_threshold {
                            self.detection_state =
                                DragDetectionState::WaitingForClickThreshold { pressed_at };
                        } else {
                            self.detection_state = DragDetectionState::Cancelled(
                                "Drag distance exceeded scroll threshold",
                            );
                        }
                    } else if !is_below_scroll_threshold {
                        self.detection_state = DragDetectionState::Cancelled(
                            "Drag distance exceeded scroll threshold",
                        );
                    }
                }
                if let DragDetectionState::WaitingForClickThreshold { pressed_at } =
                    self.detection_state
                {
                    if pressed_at.elapsed().unwrap_or_default() >= config.click_tolerance_timeout {
                        self.detection_state = DragDetectionState::CouldBeValidDrag;
                    }
                }
            }
        });

        let pointer_pos = ui
            .input(|i| i.pointer.hover_pos())
            .or_else(|| self.detection_state.last_pointer_pos());

        let dragged_item_rect = if let DragDetectionState::Dragging {
            offset,
            dragged_item_size,
            ..
        } = &self.detection_state
        {
            Some(Rect::from_min_size(
                pointer_pos.unwrap_or_default() + *offset,
                *dragged_item_size,
            ))
        } else {
            None
        };

        let mut item_iter = ItemIterator::new(self, dragged_item_rect, *ui.layout());
        callback(ui, &mut item_iter);

        let ItemIterator {
            source_item,
            hovering_over_any_handle,
            mut closest_item,
            mark_next_as_closest_item,
            last_item,
            hovering_last_item,
            ..
        } = item_iter;

        // This is only some if we're hoving over the last item
        let hovering_last_item = if mark_next_as_closest_item.is_some() {
            closest_item = Some((0.0, last_item));
            // We're only doing this once or we wouldn't be able to move back to the
            // second to last item
            !hovering_last_item
        } else {
            false
        };

        let pointer_released = ui.input(|i| i.pointer.any_released());
        let should_update = closest_item.is_some_and(|i| i.1.is_some());

        // The cursor is not hovering over any item, so cancel
        if first_frame && !hovering_over_any_handle {
            self.detection_state =
                DragDetectionState::Cancelled("Cursor not hovering over any item handle");
        }

        let drag_phase_changed_this_frame = false;

        let hovering_item = closest_item.and_then(|i| i.1);

        if let DragDetectionState::Dragging {
            closest_item: closest_out,
            source_idx: source_idx_out,
            hovering_idx: hovering_idx_out,
            last_pointer_pos: last_pointer_pos_out,
            hovering_last_item: hovering_last_item_out,
            ..
        } = &mut self.detection_state
        {
            if let Some((hovering_idx, hovering_id, pos)) = hovering_item {
                *closest_out = (hovering_id, pos);
                *hovering_idx_out = hovering_idx;
                *hovering_last_item_out = hovering_last_item;
            }
            if let Some(pointer_pos) = pointer_pos {
                *last_pointer_pos_out = pointer_pos;
            }
            if let Some(source_item) = source_item {
                *source_idx_out = source_item.0;
            }
        }

        if self.detection_state.is_dragging() {
            if let Some(pointer_pos) = pointer_pos {
                // If we are in a ScrollArea, allow for scrolling while dragging
                ui.scroll_to_rect(Rect::from_center_size(pointer_pos, Vec2::splat(50.0)), None);
            }
        }

        let mut response = if drag_phase_changed_this_frame {
            DragDropResponse {
                finished: false,
                update: None,
                state: self.detection_state.clone(),
                cancellation_reason: None,
                has_changed: false,
            }
        } else if let DragDetectionState::Dragging {
            source_idx,
            hovering_idx,
            hovering_last_item,
            ..
        } = self.detection_state
        {
            DragDropResponse {
                finished: false,
                update: Some(DragUpdate {
                    from: source_idx,
                    to: if hovering_last_item {
                        hovering_idx + 1
                    } else {
                        hovering_idx
                    },
                }),
                state: self.detection_state.clone(),
                cancellation_reason: None,
                has_changed: should_update,
            }
        } else {
            DragDropResponse {
                finished: false,
                update: None,
                state: self.detection_state.clone(),
                cancellation_reason: None,
                has_changed: false,
            }
        };

        if pointer_released {
            if let Some(dragged_item) = self.detection_state.dragged_item() {
                response.finished = true;

                self.detection_state = DragDetectionState::TransitioningBackAfterDragFinished {
                    dragged_item_size: self.detection_state.dragged_item_size(),
                    id: dragged_item,
                };
            }
        }

        ui.input(|input| {
            if !input.pointer.any_down()
                && !matches!(
                    self.detection_state,
                    DragDetectionState::TransitioningBackAfterDragFinished { .. }
                )
            {
                if let DragDetectionState::Cancelled(msg) = self.detection_state {
                    response.cancellation_reason = Some(msg);
                }
                self.detection_state = DragDetectionState::None;
            }
        });

        // We are not over any target, cancel the drag
        if let DragDetectionState::CouldBeValidDrag = self.detection_state {
            self.detection_state = DragDetectionState::Cancelled("Not hovering over any target");
        }

        // Repaint continuously while we are evaluating the drag
        if self.detection_state.is_evaluating_drag() {
            ui.ctx().request_repaint();
        }

        response
    }
}
