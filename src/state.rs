use std::hash::Hash;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, SystemTime};

use egui::{CursorIcon, Id, InnerResponse, LayerId, Order, Pos2, Rect, Sense, Ui, Vec2};

use crate::ItemState;
#[cfg(target_arch = "wasm32")]
use web_time::{Duration, SystemTime};

use crate::utils::shift_vec;

/// Item that can be reordered using drag and drop
pub trait DragDropItem {
    /// Unique id for the item, to allow egui to keep track of its dragged state between frames
    fn id(&self) -> Id;
}

impl<T: Hash> DragDropItem for T {
    fn id(&self) -> Id {
        Id::new(self)
    }
}

/// An instruction in what order to update the source list.
/// The item at from should be removed from the list and inserted at to.
/// You can use [shift_vec] to do this for a Vec.
#[derive(Debug, Clone)]
pub struct DragUpdate {
    /// Index of the item to move
    pub from: usize,
    /// Where to move the item to
    pub to: usize,
}

/// Response containing state of the drag & drop list and a potential update to the source list.
/// The update can be applied immediately or at latest when [DragDropResponse::is_drag_finished] returns true.
#[derive(Debug, Clone)]
pub struct DragDropResponse {
    state: DragDetectionState,
    /// Contains ongoing information about which index is currently being dragged where.
    /// You can use this to consistently update the source list while the drag & drop event is ongoing.
    /// If you only want to update the source list when the drag & drop event has finished, use [DragDropResponse::final_update] instead.
    pub update: Option<DragUpdate>,
    finished: bool,
    cancellation_reason: Option<&'static str>,
}

impl DragDropResponse {
    /// Returns true if we are currently evaluating whether a drag should be started.
    pub fn is_evaluating_drag(&self) -> bool {
        self.state.is_evaluating_drag()
    }

    /// Returns true if we are currently dragging an item.
    pub fn is_dragging(&self) -> bool {
        self.state.is_dragging()
    }

    /// Returns the id of the item that is currently being dragged.
    pub fn dragged_item_id(&self) -> Option<Id> {
        self.state.dragged_item()
    }

    /// Returns true if the drag & drop event has finished and the item has been dropped.
    /// The update should be applied to the source list.
    pub fn is_drag_finished(&self) -> bool {
        self.finished
    }

    /// Utility function to update a Vec with the current drag & drop state.
    /// You can use this to consistently update the source list while the drag & drop event is ongoing.
    pub fn update_vec<T>(&self, vec: &mut [T]) {
        if let Some(update) = &self.update {
            shift_vec(update.from, update.to, vec);
        }
    }

    /// Returns the update if the drag & drop event has finished and the item has been dropped.
    /// Useful for the if let syntax.
    pub fn final_update(&self) -> Option<DragUpdate> {
        if self.finished {
            self.update.clone()
        } else {
            None
        }
    }

    /// Returns a [Option<&str>] with the reason if a drag & drop event was cancelled.
    pub fn cancellation_reason(&self) -> Option<&'static str> {
        self.cancellation_reason
    }
}

/// Holds the data needed to draw the floating item while it is being dragged
/// Deprecated: Use [crate::dnd] or [crate::Dnd::new] instead
#[derive(Clone, Debug)]
pub struct DragDropUi {
    detection_state: DragDetectionState,
    drag_animation_id_count: usize,
    /// If the mobile config is set, we will use it if we detect a touch event
    touch_config: Option<DragDropConfig>,
    mouse_config: DragDropConfig,
}

impl Default for DragDropUi {
    fn default() -> Self {
        DragDropUi {
            detection_state: DragDetectionState::None,
            drag_animation_id_count: 0,
            touch_config: Some(DragDropConfig::touch()),
            mouse_config: DragDropConfig::mouse(),
        }
    }
}

/// [Handle::ui] is used to draw the drag handle
pub struct Handle<'a> {
    id: Id,
    state: &'a mut DragDropUi,
    hovering_over_any_handle: &'a mut bool,
    sense: Option<Sense>,
    item_pos: Pos2,
}

#[derive(Debug, Default, Clone)]
enum DragDetectionState {
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
        offset: Vec2,
        phase: DragPhase,
    },
    TransitioningBackAfterDragFinished {
        id: Id,
        from: Option<Pos2>,
    },
}

#[derive(Debug, Clone)]
enum DragPhase {
    FirstFrame,
    Rest {
        // For touch devices, we need to remember the last pointer position
        // so we don't loose it during the last frame
        last_pointer_pos: Pos2,

        dragged_item_size: Vec2,

        /// This will always be set unless we are at the bottom of the list
        hovering_above_item: Option<Id>,
        /// This will be set if we are at the bottom of the list
        hovering_below_item: Option<Id>,

        // These should only be used during for output, as to not cause issues when item indexes change
        hovering_idx: usize,
        source_idx: usize,
    },
}

impl DragPhase {
    fn is_first_frame(&self) -> bool {
        matches!(self, DragPhase::FirstFrame)
    }
}

impl DragDetectionState {
    fn is_evaluating_drag(&self) -> bool {
        matches!(self, DragDetectionState::WaitingForClickThreshold { .. })
            || matches!(self, DragDetectionState::PressedWaitingForDelay { .. })
            || matches!(self, DragDetectionState::CouldBeValidDrag)
    }

    fn is_dragging(&self) -> bool {
        matches!(self, DragDetectionState::Dragging { .. })
    }

    fn dragged_item(&self) -> Option<Id> {
        match self {
            DragDetectionState::Dragging { id, .. } => Some(*id),
            _ => None,
        }
    }

    fn is_dragging_item(&self, id: Id) -> bool {
        self.dragged_item() == Some(id)
    }

    fn offset(&self) -> Option<Vec2> {
        match self {
            DragDetectionState::Dragging { offset, .. } => Some(*offset),
            _ => None,
        }
    }

    fn dragged_item_size(&self) -> Option<Vec2> {
        match self {
            DragDetectionState::Dragging {
                phase: DragPhase::Rest {
                    dragged_item_size, ..
                },
                ..
            } => Some(*dragged_item_size),
            _ => None,
        }
    }

    fn hovering_item(&self) -> Option<Id> {
        match self {
            DragDetectionState::Dragging {
                phase:
                    DragPhase::Rest {
                        hovering_above_item: hovering_item,
                        hovering_below_item,
                        ..
                    },
                ..
            } => hovering_item.or(*hovering_below_item),
            _ => None,
        }
    }

    fn hovering_below_item(&self) -> Option<Id> {
        match self {
            DragDetectionState::Dragging {
                phase:
                    DragPhase::Rest {
                        hovering_below_item,
                        ..
                    },
                ..
            } => *hovering_below_item,
            _ => None,
        }
    }

    fn last_pointer_pos(&self) -> Option<Pos2> {
        match self {
            DragDetectionState::Dragging {
                phase: DragPhase::Rest {
                    last_pointer_pos, ..
                },
                ..
            } => Some(*last_pointer_pos),
            _ => None,
        }
    }
}

impl<'a> Handle<'a> {
    /// You can add [Sense::click] if you want to listen for clicks on the handle
    /// **Warning**: This will make anything sensing clicks in the handle not draggable
    /// Make sure to not set this if your handle consists of a single button, and directly
    /// query the button for clicks.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.sense = Some(sense);
        self
    }

    /// Draw the drag handle. Use [Handle::sense] to add a click sense.
    /// You can also add buttons in the handle, but they won't be interactive if you pass Sense::click
    pub fn ui(self, ui: &mut Ui, contents: impl FnOnce(&mut Ui)) -> egui::Response {
        let u = ui.scope(contents);

        let response = if let Some(sense) = self.sense {
            u.response.interact(sense)
        } else {
            u.response
        };

        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::Grab);
            *self.hovering_over_any_handle = true;
        }

        let offset = self.item_pos.to_vec2() - response.hover_pos().unwrap_or_default().to_vec2();

        let drag_distance = ui.input(|i| {
            (i.pointer.hover_pos().unwrap_or_default()
                - i.pointer.press_origin().unwrap_or_default())
            .length()
        });

        let click_threshold = 1.0;
        let is_above_click_threshold = drag_distance > click_threshold;

        if response.hovered()
            && response
                .rect
                .contains(ui.input(|input| input.pointer.press_origin().unwrap_or_default()))
        {
            if let DragDetectionState::WaitingForClickThreshold { pressed_at } =
                self.state.detection_state
            {
                // It should be save to stop anything else being dragged here
                // This is important so any ScrollArea isn't being dragged while we wait for the click threshold
                ui.memory_mut(|mem| mem.stop_dragging());
                if is_above_click_threshold
                    || pressed_at.elapsed().unwrap_or_default()
                        > self.state.config(ui).click_tolerance_timeout
                {
                    self.state.detection_state = DragDetectionState::CouldBeValidDrag;
                }
            }
        };

        if response.hovered()
            && matches!(
                self.state.detection_state,
                DragDetectionState::CouldBeValidDrag
            )
        {
            self.state.detection_state = DragDetectionState::Dragging {
                id: self.id,
                offset,
                phase: DragPhase::FirstFrame,
            };
            self.state.drag_animation_id_count += 1;
            ui.memory_mut(|mem| mem.set_dragged_id(self.id));
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
    /// How far can the pointer move during the [DragDropConfig::drag_delay] before the drag is cancelled?
    pub scroll_tolerance: Option<f32>,
    /// How far does the pointer have to move before a drag starts?
    /// This is useful when the handle is also a button.
    /// If the pointer is released before this threshold, the drag never starts and the button / handle can be clicked.
    /// If you want to detect clicks on the handle itself, [Handle::sense] to add a click sense to the handle.
    pub click_tolerance: f32,
    /// If we have been holding longer than this duration, a drag will be started even if the pointer has not moved above [DragDropConfig::click_tolerance].
    pub click_tolerance_timeout: Duration,
}

impl Default for DragDropConfig {
    fn default() -> Self {
        Self::mouse()
    }
}

impl DragDropConfig {
    /// Optimized for mouse usage
    pub fn mouse() -> Self {
        Self {
            click_tolerance: 1.0,
            drag_delay: Duration::from_millis(0),
            scroll_tolerance: None,
            click_tolerance_timeout: Duration::from_millis(250),
        }
    }

    /// Optimized for touch usage in a fixed size area (no scrolling)
    /// Has a higher click tolerance than [DragDropConfig::mouse]
    pub fn touch() -> Self {
        Self {
            scroll_tolerance: None,
            click_tolerance: 3.0,
            drag_delay: Duration::from_millis(0),
            click_tolerance_timeout: Duration::from_millis(250),
        }
    }

    /// Optimized for touch usage in a scrollable area
    pub fn touch_scroll() -> Self {
        Self {
            scroll_tolerance: Some(6.0),
            click_tolerance: 3.0,
            drag_delay: Duration::from_millis(300),
            click_tolerance_timeout: Duration::from_millis(250),
        }
    }
}

/// [DragDropUi] stores the state of the Drag & Drop list.
/// # Example
/// ```rust;no_run
/// use egui_dnd::DragDropUi;
/// use eframe::App;
/// use eframe::egui::Context;
/// use eframe::Frame;
/// use eframe::egui::CentralPanel;
/// use egui_dnd::utils::shift_vec;
///
/// struct DnDApp {
///     items: Vec<String>,
///     dnd: DragDropUi,
/// }
///
///
/// impl App for DnDApp {
///     fn update(&mut self, ctx: &Context, frame: &mut Frame) {
///         CentralPanel::default().show(ctx, |ui| {
///             let response = self.dnd.ui(ui, self.items.iter_mut(), |item, ui, handle, dragging| {
///                 ui.horizontal(|ui| {
///                     handle.ui(ui, |ui| {
///                         ui.label("grab");
///                     });
///                     ui.label(item.clone());
///                 });
///             });
///             if let Some(response) = response.final_update() {
///                 shift_vec(response.from, response.to, &mut self.items);
///             }
///         });
///     }
/// }
///
/// use eframe::NativeOptions;
/// let dnd = DragDropUi::default();
/// eframe::run_native("DnD Example", NativeOptions::default(), Box::new(|_| {
///     Box::new(DnDApp {
///         dnd: DragDropUi::default(),
///         items: vec!["a", "b", "c"].into_iter().map(|s| s.to_string()).collect(),
///     })
/// }));
/// ```
impl DragDropUi {
    /// Sets the config used when dragging with the mouse or when no touch config is set
    pub fn with_mouse_config(mut self, config: DragDropConfig) -> Self {
        self.mouse_config = config;
        self
    }

    /// Sets the config used when dragging with touch
    /// If None, the mouse config is used instead
    /// Use [DragDropConfig::touch] or [DragDropConfig::touch_scroll] to get a config optimized for touch
    /// The default is [DragDropConfig::touch]
    /// For dragging in a ScrollArea, use [DragDropConfig::touch_scroll]
    pub fn with_touch_config(mut self, config: Option<DragDropConfig>) -> Self {
        self.touch_config = config;
        self
    }

    fn config(&self, ui: &Ui) -> &DragDropConfig {
        if ui.input(|i| i.any_touches()) {
            self.touch_config.as_ref().unwrap_or(&self.mouse_config)
        } else {
            &self.mouse_config
        }
    }

    /// Draw the items and handle drag & drop stuff
    pub fn ui<T: DragDropItem>(
        &mut self,
        ui: &mut Ui,
        values: impl Iterator<Item = T>,
        mut item_ui: impl FnMut(T, &mut Ui, Handle, ItemState),
    ) -> DragDropResponse {
        // During the first frame, we check if the pointer is actually over any of the item handles and cancel the drag if it isn't
        let mut first_frame = false;
        let config = self.config(ui).clone();

        let dnd_animation_id = Id::new(self.drag_animation_id_count);

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

        let dragged_item_pos =
            pointer_pos.unwrap_or_default() + self.detection_state.offset().unwrap_or_default();
        let dragged_item_rect = Rect::from_min_size(
            dragged_item_pos,
            self.detection_state.dragged_item_size().unwrap_or_default(),
        );
        let dragged_item_center = dragged_item_rect.center();
        let mut above_item = None;
        let mut below_item = None;

        let mut should_add_space_at_end = true;

        let mut source_item = None;
        let mut dragged_item_size = None;

        let mut add_space_for_previous_item = false;

        let mut hovering_over_any_handle = false;

        ui.scope(|ui| {
            let item_spacing = ui.spacing().item_spacing.y;
            ui.spacing_mut().item_spacing.y = 0.0;

            // In egui, if the value changes during animation, we start at 0 or 1 again instead of returning from the current value.
            // This causes flickering, we use the animation budget to mitigate this (Stops the total value of animations to be > 1).
            let mut animation_budget = 1.0;

            DragDropUi::drop_target(ui, true, |ui| {
                values.enumerate().for_each(|(idx, item)| {
                    let item_id = item.id();
                    let is_dragged_item = self.detection_state.is_dragging_item(item_id);

                    let hovering_this_item = self.detection_state.hovering_item() == Some(item_id);
                    let mut add_space = hovering_this_item;
                    if add_space_for_previous_item {
                        add_space = true;
                        add_space_for_previous_item = false;
                    }
                    if add_space
                        && (is_dragged_item
                            || self.detection_state.hovering_below_item() == Some(item_id))
                    {
                        add_space_for_previous_item = true;
                        add_space = false;
                    }
                    if add_space {
                        should_add_space_at_end = false;
                    }

                    let animation_id = Id::new(item_id)
                        .with("dnd_space_animation")
                        .with(dnd_animation_id);

                    let mut x = ui.ctx().animate_bool(animation_id, add_space);

                    let space = dragged_item_rect.height() + item_spacing;
                    if x > 0.0 {
                        x = x.min(animation_budget);
                        ui.allocate_space(Vec2::new(0.0, space * x));
                    }
                    animation_budget -= x;

                    // Add normal item spacing
                    if !self.detection_state.is_dragging_item(item_id) {
                        ui.add_space(item_spacing);
                    }

                    let rect = ui
                        .scope(|ui| {
                            // Restore spacing so it doesn't affect inner ui
                            ui.style_mut().spacing.item_spacing.y = item_spacing;
                            self.drag_source(
                                ui,
                                item_id,
                                &mut hovering_over_any_handle,
                                |ui, handle| {
                                    item_ui(
                                        item,
                                        ui,
                                        handle,
                                        ItemState {
                                            dragged: is_dragged_item,
                                            index: idx,
                                        },
                                    );
                                },
                            )
                        })
                        .inner;

                    // TODO: Use .top and .bottom here for more optimistic switching
                    if dragged_item_center.y < rect.center().y && above_item.is_none() {
                        above_item = Some((idx, item_id));
                    }
                    if dragged_item_center.y > rect.center().y {
                        below_item = Some((idx, item_id));
                    }

                    if self.detection_state.is_dragging_item(item_id) {
                        source_item = Some((idx, item_id));
                        dragged_item_size = Some(rect.size());
                    }
                });
            });

            let mut x = ui.ctx().animate_bool(
                Id::new("dnd_end_space").with(dnd_animation_id),
                should_add_space_at_end && self.detection_state.hovering_item().is_some(),
            );
            x = x.min(animation_budget);
            if x > 0.0 {
                let space = dragged_item_rect.height() + item_spacing;
                ui.allocate_exact_size(Vec2::new(0.0, space * x), Sense::hover());
            }
        });

        // The cursor is not hovering over any item, so cancel
        if first_frame && !hovering_over_any_handle {
            self.detection_state =
                DragDetectionState::Cancelled("Cursor not hovering over any item handle");
        }

        let hovering_item = above_item;
        if let DragDetectionState::Dragging { phase, .. } = &mut self.detection_state {
            if let Some(source_idx) = source_item {
                if let Some(dragged_item_size) = dragged_item_size {
                    if let DragPhase::FirstFrame = phase {
                        // Prevent flickering
                        self.drag_animation_id_count += 1;
                    }
                    let hovering_item_id = hovering_item.map(|i| i.1);

                    *phase = DragPhase::Rest {
                        last_pointer_pos: pointer_pos.unwrap_or_default(),
                        dragged_item_size,
                        hovering_above_item: hovering_item_id,
                        hovering_below_item: below_item.map(|i| i.1),
                        hovering_idx: hovering_item
                            .map(|i| i.0)
                            .or(below_item.map(|i| i.0 + 1))
                            .unwrap_or_default(), // One of these must be Some
                        source_idx: source_idx.0,
                    };
                }
            }
        }

        let mut response = if let DragDetectionState::Dragging {
            id,
            phase:
                DragPhase::Rest {
                    hovering_idx,
                    source_idx,
                    ..
                },
            ..
        } = self.detection_state
        {
            let mut response = DragDropResponse {
                finished: false,
                update: Some(DragUpdate {
                    from: source_idx,
                    to: hovering_idx,
                }),
                state: self.detection_state.clone(),
                cancellation_reason: None,
            };

            if ui.input(|i| i.pointer.any_released()) {
                response.finished = true;
                self.drag_animation_id_count += 1;

                self.detection_state = DragDetectionState::TransitioningBackAfterDragFinished {
                    from: Some(dragged_item_pos),
                    id,
                };
            }

            response
        } else {
            DragDropResponse {
                finished: false,
                update: None,
                state: self.detection_state.clone(),
                cancellation_reason: None,
            }
        };

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

    fn drag_source(
        &mut self,
        ui: &mut Ui,
        id: Id,
        hovering_over_any_handle: &mut bool,
        drag_body: impl FnOnce(&mut Ui, Handle),
    ) -> Rect {
        let last_pointer_pos = self.detection_state.last_pointer_pos();
        if let DragDetectionState::Dragging {
            id: dragging_id,
            offset,
            phase,
            ..
        } = &mut self.detection_state
        {
            // Draw the item item in it's original position in the first frame to avoid flickering
            if id == *dragging_id && !phase.is_first_frame() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::Grabbing);

                let _layer_id = LayerId::new(Order::Tooltip, id);

                let pointer_pos = ui
                    .ctx()
                    .pointer_hover_pos()
                    .or(last_pointer_pos)
                    .unwrap_or_else(|| ui.next_widget_position());
                let position = pointer_pos + *offset;

                // If we are in a ScrollArea, allow for scrolling while dragging
                ui.scroll_to_rect(
                    Rect::from_center_size(pointer_pos, Vec2::splat(100.0)),
                    None,
                );

                let InnerResponse { inner: rect, .. } = self.draw_floating_at_position(
                    ui,
                    id,
                    position,
                    hovering_over_any_handle,
                    drag_body,
                );

                return Rect::from_min_size(ui.next_widget_position(), rect.size());
            }
        } else if let DragDetectionState::TransitioningBackAfterDragFinished {
            from,
            id: transitioning_id,
        } = &mut self.detection_state
        {
            if id == *transitioning_id {
                let value = std::mem::take(from).unwrap_or(ui.next_widget_position());
                let time = ui.style().animation_time;
                let x = ui.ctx().animate_value_with_time(
                    id.with("transitioning_back_x")
                        .with(self.drag_animation_id_count),
                    value.x,
                    time,
                );
                let y = ui.ctx().animate_value_with_time(
                    id.with("transitioning_back_y")
                        .with(self.drag_animation_id_count),
                    value.y,
                    time,
                );
                let position = Pos2::new(x, y);
                if position == ui.next_widget_position() {
                    // Animation finished
                    self.detection_state = DragDetectionState::None;
                }

                let InnerResponse { inner: rect, .. } = self.draw_floating_at_position(
                    ui,
                    id,
                    position,
                    hovering_over_any_handle,
                    drag_body,
                );
                return ui.allocate_exact_size(rect.size(), Sense::hover()).0;
            }
        }

        let pos = ui.next_widget_position();

        let scope = ui.scope(|ui| {
            drag_body(
                ui,
                Handle {
                    item_pos: pos,
                    state: self,
                    id,
                    hovering_over_any_handle,
                    sense: None,
                },
            )
        });
        scope.response.rect
    }

    fn draw_floating_at_position(
        &mut self,
        ui: &mut Ui,
        id: Id,
        pos: Pos2,
        hovering_over_any_handle: &mut bool,
        body: impl FnOnce(&mut Ui, Handle),
    ) -> InnerResponse<Rect> {
        let _layer_id = LayerId::new(Order::Tooltip, id);

        egui::Area::new("draggable_item")
            .interactable(false)
            .fixed_pos(pos)
            .show(ui.ctx(), |ui| {
                ui.scope(|ui| {
                    body(
                        ui,
                        Handle {
                            item_pos: pos,
                            state: self,
                            id,
                            hovering_over_any_handle,
                            sense: None,
                        },
                    )
                })
                .response
                .rect
            })
    }

    fn drop_target<R>(
        ui: &mut Ui,
        _can_accept_what_is_being_dragged: bool,
        body: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        let margin = Vec2::splat(4.0);

        let outer_rect_bounds = ui.available_rect_before_wrap();
        let inner_rect = outer_rect_bounds.shrink2(margin);

        let mut content_ui = ui.child_ui(inner_rect, *ui.layout());

        let ret = body(&mut content_ui);
        let outer_rect =
            Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
        let (_rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

        InnerResponse::new(ret, response)
    }
}
