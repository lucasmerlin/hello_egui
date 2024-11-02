#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod flex_widget;

pub use crate::flex_widget::FlexWidget;

use egui::{
    Align, Align2, Direction, Frame, Id, InnerResponse, Layout, Margin, Pos2, Rect, Response,
    Sense, Ui, UiBuilder, Vec2, Widget,
};
use std::mem;

/// The direction in which the flex container should lay out its children.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FlexDirection {
    #[default]
    Horizontal,
    Vertical,
}

/// How to justify the content (alignment in the main axis).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FlexJustify {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// How to align the content in the cross axis on the current line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FlexAlign {
    Start,
    End,
    Center,
    #[default]
    Stretch,
}

/// How to align the content in the cross axis across the whole container.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum FlexAlignContent {
    Start,
    End,
    Center,
    #[default]
    Stretch,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Points(f32),
    Percent(f32),
}

impl From<f32> for Size {
    fn from(p: f32) -> Self {
        Size::Points(p)
    }
}

impl Size {
    pub fn get(&self, total: f32) -> f32 {
        match self {
            Size::Points(p) => *p,
            Size::Percent(p) => total * *p,
        }
    }
}

/// A flex container.
#[derive(Debug, Clone, PartialEq)]
pub struct Flex {
    id_salt: Option<Id>,
    direction: FlexDirection,
    justify: FlexJustify,
    align_content: FlexAlignContent,
    gap: Option<Vec2>,
    default_item: FlexItem,
    wrap: bool,
    width: Option<Size>,
    height: Option<Size>,
}

impl Default for Flex {
    fn default() -> Self {
        Self {
            id_salt: None,
            direction: FlexDirection::default(),
            justify: FlexJustify::default(),
            align_content: FlexAlignContent::default(),
            gap: None,
            default_item: FlexItem::default(),
            wrap: false,
            width: None,
            height: None,
        }
    }
}

/// Configuration for a flex item.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FlexItem {
    grow: Option<f32>,
    basis: Option<f32>,
    align_self: Option<FlexAlign>,
    align_content: Option<Align2>,
    shrink: bool,
}

/// Create a new flex item. Shorthand for [`FlexItem::default`].
pub fn item() -> FlexItem {
    FlexItem::default()
}

impl FlexItem {
    /// Create a new flex item. You can also use the [`item`] function.
    pub fn new() -> Self {
        Self::default()
    }

    /// How much should this item grow compared to the other items.
    ///
    /// By default items don't grow.
    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = Some(grow);
        self
    }

    /// Set the default size of the item, before it grows.
    /// If this is not set, the items "intrinsic size" will be used.
    pub fn basis(mut self, basis: f32) -> Self {
        self.basis = Some(basis);
        self
    }

    /// How do we align the item in the cross axis?
    ///
    /// Default is `stretch`.
    pub fn align_self(mut self, align_self: FlexAlign) -> Self {
        self.align_self = Some(align_self);
        self
    }

    /// If `align_self` is stretch, how do we align the content?
    ///
    /// Default is `center`.
    pub fn align_self_content(mut self, align_self_content: Align2) -> Self {
        self.align_content = Some(align_self_content);
        self
    }

    /// Shrink this item if there isn't enough space.
    ///
    /// Note: You may only ever set this on a single item in a flex container.
    pub fn shrink(mut self) -> Self {
        self.shrink = true;
        self
    }
}

impl Flex {
    /// Create a new flex container.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new horizontal flex container.
    pub fn horizontal() -> Self {
        Self::default().direction(FlexDirection::Horizontal)
    }

    /// Create a new vertical flex container.
    pub fn vertical() -> Self {
        Self::default().direction(FlexDirection::Vertical)
    }

    /// Set the direction of the flex container.
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set how to justify the content (alignment in the main axis).
    /// This will only have an effect if all items have grow set to 0.
    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.justify = justify;
        self
    }

    /// Set the default configuration for the items in the flex container.
    pub fn align_items(mut self, align_items: FlexAlign) -> Self {
        self.default_item.align_self = Some(align_items);
        self
    }

    /// If `align_items` is stretch, how do we align the item content?
    pub fn align_items_content(mut self, align_item_content: Align2) -> Self {
        self.default_item.align_content = Some(align_item_content);
        self
    }

    /// Set how to align the content in the cross axis across the whole container.
    ///
    /// This only has an effect if wrap is set to true.
    ///
    /// Default is `stretch`.
    pub fn align_content(mut self, align_content: FlexAlignContent) -> Self {
        self.align_content = align_content;
        self
    }

    /// Set the default grow factor for the items in the flex container.
    pub fn grow_items(mut self, grow: f32) -> Self {
        self.default_item.grow = Some(grow);
        self
    }

    /// Set the gap between the items in the flex container.
    ///
    /// Default is `item_spacing` of the [`Ui`].
    pub fn gap(mut self, gap: Vec2) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Should the flex container wrap it's content.
    /// If this is set to `false` the content may overflow the [`Ui::max_rect`]
    ///
    /// Default: false
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    /// Customize the id of the flex container to prevent conflicts with other flex containers.
    pub fn id_salt(mut self, id_salt: impl Into<Id>) -> Self {
        self.id_salt = Some(id_salt.into());
        self
    }

    /// Set the width of the flex container in points (pixels).
    ///
    /// The default depends on the parents horizontal justify.
    /// If `ui.layout().horizontal_justify()` is:
    /// - true, the width will be set to 100%.
    /// - false, the width will depend on the width of the content.
    pub fn width(mut self, width: impl Into<Size>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// Set the height of the flex container in points (pixels).
    ///
    /// The default depends on the parents vertical justify.
    /// If `ui.layout().vertical_justify()` is:
    /// - true, the height will be set to 100%.
    /// - false, the height will depend on the height of the content.
    pub fn height(mut self, height: impl Into<Size>) -> Self {
        self.height = Some(height.into());
        self
    }

    /// Set the width of the flex container as a percentage of the available width.
    /// 1.0 means 100%.
    ///
    /// Check [`Self::width`] for more info.
    pub fn width_percent(mut self, width: f32) -> Self {
        self.width = Some(Size::Percent(width));
        self
    }

    /// Set the height of the flex container as a percentage of the available height.
    /// 1.0 means 100%.
    ///
    /// Check [`Self::height`] for more info.
    pub fn height_percent(mut self, height: f32) -> Self {
        self.height = Some(Size::Percent(height));
        self
    }

    /// Set the size of the flex container in points (pixels).
    ///
    /// Check [`Self::width`] and [`Self::height`] for more info.
    pub fn size(mut self, size: impl Into<Vec2>) -> Self {
        let size = size.into();
        self.width = Some(Size::Points(size.x));
        self.height = Some(Size::Points(size.y));
        self
    }

    /// Set the width of the flex container to 100%.
    pub fn w_full(mut self) -> Self {
        self.width = Some(Size::Percent(1.0));
        self
    }

    /// Set the height of the flex container to 100%.
    pub fn h_full(mut self) -> Self {
        self.height = Some(Size::Percent(1.0));
        self
    }

    /// The width of the flex container will be set to the width of the content, but not larger
    /// than the available width (unless wrap is set to false).
    ///
    /// This is the default.
    pub fn w_auto(mut self) -> Self {
        self.width = None;
        self
    }

    /// The height of the flex container will be set to the height of the content, but not larger
    /// than the available height (unless wrap is set to false).
    ///
    /// This is the default.
    pub fn h_auto(mut self) -> Self {
        self.height = None;
        self
    }

    #[track_caller]
    #[allow(clippy::too_many_lines)]
    fn show_inside<R>(
        mut self,
        ui: &mut Ui,
        target_size: Option<Vec2>,
        max_item_size: Option<Vec2>,
        f: impl FnOnce(&mut FlexInstance) -> R,
    ) -> (Vec2, InnerResponse<R>) {
        let id = if let Some(id_salt) = self.id_salt {
            ui.id().with(id_salt)
        } else {
            ui.auto_id_with("flex")
        };
        let previous_state: FlexState = ui
            .ctx()
            .memory(|mem| mem.data.get_temp(id).clone().unwrap_or_default());

        let frame_time = ui.ctx().input(|i| i.time);
        let passes = ui.ctx().cumulative_pass_nr();
        if cfg!(debug_assertions)
            && (previous_state.frame_time == frame_time && previous_state.passes == passes)
        {
            panic!("Id clash in flex container! Id: {id:?}");
        }

        let width = self.width.or_else(|| {
            if ui.layout().horizontal_justify() {
                Some(Size::Percent(1.0))
            } else {
                None
            }
        });
        let height = self.height.or_else(|| {
            if ui.layout().vertical_justify() {
                Some(Size::Percent(1.0))
            } else {
                None
            }
        });

        let layout = match self.direction {
            FlexDirection::Horizontal => Layout::left_to_right(Align::Min),
            FlexDirection::Vertical => Layout::top_down(Align::Min),
        };

        let mut state_changed = false;

        let parent_rect = ui.max_rect();

        let r = ui.scope_builder(
            UiBuilder::new()
                .layout(layout)
                .max_rect(round_rect(ui.available_rect_before_wrap())),
            |ui| {
                let gap = self.gap.unwrap_or(ui.spacing_mut().item_spacing);
                let original_item_spacing = mem::replace(&mut ui.spacing_mut().item_spacing, gap);

                // We ceil in order to prevent rounding errors to wrap the layout unexpectedly
                let available_size = target_size.unwrap_or(ui.available_size()).ceil();

                // TODO: Is this right? I would expect Vec2::min...
                let size_origin = Vec2::max(
                    target_size.unwrap_or(parent_rect.size()),
                    parent_rect.size(),
                );
                // let size_origin = parent_rect.size();

                let size = [
                    width.map(|w| round(w.get(size_origin.x))),
                    height.map(|h| round(h.get(size_origin.y))),
                ];

                let direction = usize::from(!ui.layout().main_dir().is_horizontal());
                let cross_direction = 1 - direction;

                // TODO: I think it should be possible to cache the layout
                let rows = self.layout_rows(
                    &previous_state,
                    available_size,
                    size,
                    gap,
                    direction,
                    ui.min_rect().min,
                );

                let max_item_size = round_vec2(max_item_size.unwrap_or(available_size));

                let mut instance = FlexInstance {
                    current_row: 0,
                    current_row_index: 0,
                    flex: &self,
                    state: FlexState {
                        items: vec![],
                        max_item_size,
                        frame_time,
                        passes,
                    },
                    direction,
                    row_ui: FlexInstance::row_ui(ui, rows.first()),
                    ui,
                    rows,
                    max_item_size,
                    last_max_item_size: previous_state.max_item_size,
                    item_spacing: original_item_spacing,
                    size,
                };

                let r = f(&mut instance);

                let mut min_size =
                    instance
                        .state
                        .items
                        .iter()
                        .fold(Vec2::ZERO, |mut current, item| {
                            current[direction] += item.min_size_with_margin()[direction];
                            current[cross_direction] = f32::max(
                                current[cross_direction],
                                item.min_size_with_margin()[cross_direction],
                            );
                            current
                        });
                min_size[direction] += gap[direction] * (instance.state.items.len() as f32 - 1.0);

                // TODO: We should be able to calculate the min_size by looking at the rows at the
                // max item size, but form some reason this doesn't work correctly
                // This would fix wrapping in nested flexes
                // let min_size = min_size_rows.iter().fold(Vec2::ZERO, |mut current, row| {
                //     current[direction] = f32::max(current[direction], row.total_size);
                //     current[cross_direction] += row.cross_size;
                //     current
                // });

                if (&previous_state.items, &previous_state.max_item_size)
                    != (&instance.state.items, &instance.state.max_item_size)
                {
                    state_changed = true;
                }

                instance.ui.ctx().memory_mut(|mem| {
                    mem.data.insert_temp(id, instance.state);
                });

                instance.rows.iter().for_each(|row| {
                    if let Some(final_rect) = row.final_rect {
                        instance.ui.allocate_rect(final_rect, Sense::hover());
                    }
                });
                (min_size, r)
            },
        );

        // We move this down here because `#[track_caller]` doesn't work with closures
        if state_changed {
            ui.ctx()
                .request_discard("Flex item added / removed / size changed");
            ui.ctx().request_repaint();
        }

        (r.inner.0, InnerResponse::new(r.inner.1, r.response))
    }

    #[allow(clippy::too_many_lines)]
    fn layout_rows(
        &mut self,
        state: &FlexState,
        available_size: Vec2,
        size: [Option<f32>; 2],
        gap: Vec2,
        direction: usize,
        min_position: Pos2,
    ) -> Vec<RowData> {
        let cross_direction = 1 - direction;

        let available_length = size[direction].unwrap_or(available_size[direction]);
        let gap_direction = gap[direction];

        let mut rows = vec![];
        let mut current_row = RowData::default();

        let mut shrink_index = None;

        for (idx, item) in state.items.iter().enumerate() {
            if item.config.shrink && !self.wrap {
                debug_assert!(
                    shrink_index.is_none(),
                    "Only one item may have shrink set to true"
                );
                shrink_index = Some(idx);
            }

            let item_length = item
                .config
                .basis
                .map_or(item.min_size_with_margin()[direction], |basis| {
                    basis + item.margin.sum()[direction]
                });

            if item_length + gap_direction + current_row.total_size > available_length
                && !current_row.items.is_empty()
                && self.wrap
            {
                rows.push(mem::take(&mut current_row));
            }

            current_row.total_size += item_length;
            if !current_row.items.is_empty() {
                current_row.total_size += gap_direction;
            }
            current_row.total_grow += item.config.grow.unwrap_or(0.0);
            current_row.items.push(item.clone());
            if item.min_size_with_margin()[cross_direction] > current_row.cross_size {
                current_row.cross_size = item.min_size_with_margin()[cross_direction];
            }
        }

        if !current_row.items.is_empty() {
            rows.push(current_row);
        }

        let target_cross_size = size[cross_direction];
        let total_cross_size = rows.iter().map(|row| row.cross_size).sum::<f32>()
            + (rows.len().max(1) - 1) as f32 * gap[cross_direction];
        let extra_cross_space = target_cross_size.map_or(0.0, |target_cross_size| {
            f32::max(target_cross_size - total_cross_size, 0.0)
        });

        let mut extra_cross_gap_start = 0.0;
        let mut extra_cross_gap = 0.0;
        let mut extra_cross_gap_end = 0.0; // TODO: How to handle extra end space?
        let mut extra_cross_space_per_row = 0.0;

        if self.wrap == false {
            self.align_content = FlexAlignContent::Stretch;
        }
        match self.align_content {
            FlexAlignContent::Start => {
                extra_cross_gap_end = extra_cross_space;
            }
            FlexAlignContent::Stretch => {
                extra_cross_space_per_row = extra_cross_space / rows.len() as f32;
            }
            FlexAlignContent::End => {
                extra_cross_gap_start = extra_cross_space;
            }
            FlexAlignContent::Center => {
                extra_cross_gap_start = extra_cross_space / 2.0;
                extra_cross_gap_end = extra_cross_space / 2.0;
            }
            FlexAlignContent::SpaceBetween => {
                extra_cross_gap = extra_cross_space / (rows.len() as f32 - 1.0);
            }
            FlexAlignContent::SpaceAround => {
                extra_cross_gap = extra_cross_space / rows.len() as f32;
                extra_cross_gap_start = extra_cross_gap / 2.0;
                extra_cross_gap_end = extra_cross_gap / 2.0;
            }
        };

        let mut row_position = min_position;

        row_position[cross_direction] += extra_cross_gap_start;

        let row_count = rows.len();
        for (idx, row) in &mut rows.iter_mut().enumerate() {
            let mut row_size = Vec2::ZERO;
            row_size[direction] = available_length;
            row_size[cross_direction] = row.cross_size + extra_cross_space_per_row;
            // TODO: Should there be an option to also limit in the cross dir?
            // if size[cross_direction].is_some() {
            //     row_size[cross_direction] =
            //         f32::min(row_size[cross_direction], available_size[cross_direction]);
            // }

            row.cross_size_with_extra_space = row_size[cross_direction];
            row.rect = Some(Rect::from_min_size(row_position, row_size));

            row_position[cross_direction] +=
                row_size[cross_direction] + gap[cross_direction] + extra_cross_gap;

            let diff = available_length - row.total_size;
            // Only grow items if a explicit size is set or if we wrapped
            // If diff is < 0.0, we also set extra_space so we can shrink
            if (size[direction].is_some() || row_count > 1 || diff < 0.0) {
                row.extra_space = diff;
            }
            if row.total_grow == 0.0 && row.extra_space > 0.0
                // If size is none, the flex container should be sized based on the content and
                // justify doesn't apply
                && size[direction].is_some()
            {
                match self.justify {
                    FlexJustify::Start => {}
                    FlexJustify::End => {
                        row.extra_start_gap = row.extra_space;
                    }
                    FlexJustify::Center => {
                        row.extra_start_gap = row.extra_space / 2.0;
                    }
                    FlexJustify::SpaceBetween => {
                        row.extra_gap = row.extra_space / (row.items.len() as f32 - 1.0);
                    }
                    FlexJustify::SpaceAround => {
                        row.extra_gap = row.extra_space / row.items.len() as f32;
                        row.extra_start_gap = row.extra_gap / 2.0;
                    }
                    FlexJustify::SpaceEvenly => {
                        row.extra_gap = row.extra_space / (row.items.len() as f32 + 1.0);
                        row.extra_start_gap = row.extra_gap;
                    }
                }
                row.extra_gap = round(row.extra_gap).max(0.0);
                row.extra_start_gap = round(row.extra_start_gap).max(0.0);
            }
        }
        rows
    }

    /// Show the flex ui. If [`Self::wrap`] is `true`, it will try to stay within [`Ui::max_rect`].
    ///
    /// Note: You will likely get weird results when showing this within a `Ui::horizontal` layout,
    /// since it limits the `max_rect` to some small value. Use `Ui::horizontal_top` instead.
    #[track_caller]
    pub fn show<R>(self, ui: &mut Ui, f: impl FnOnce(&mut FlexInstance) -> R) -> InnerResponse<R> {
        self.show_inside(ui, None, None, f).1
    }
}

#[derive(Debug, Clone, Default)]
struct RowData {
    items: Vec<ItemState>,
    total_size: f32,
    total_grow: f32,
    extra_space: f32,
    extra_gap: f32,
    extra_start_gap: f32,
    cross_size: f32,
    cross_size_with_extra_space: f32,
    rect: Option<Rect>,
    final_rect: Option<Rect>,
}

#[derive(Debug, Clone, PartialEq)]
struct ItemState {
    id: Id,
    config: FlexItem,
    inner_size: Vec2,
    inner_min_size: Vec2,
    margin: Margin,
    remeasure_widget: bool,
}

impl ItemState {
    fn min_size_with_margin(&self) -> Vec2 {
        self.inner_min_size + self.margin.sum()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FlexState {
    items: Vec<ItemState>,
    max_item_size: Vec2,
    // We use this to keep track if there is a id clash.
    frame_time: f64,
    passes: u64,
}

/// An instance of a flex container, used to add items to the container.
pub struct FlexInstance<'a> {
    flex: &'a Flex,
    current_row: usize,
    current_row_index: usize,
    state: FlexState,
    ui: &'a mut Ui,
    rows: Vec<RowData>,
    direction: usize,
    row_ui: Ui,
    max_item_size: Vec2,
    last_max_item_size: Vec2,
    // Original item spacing to store when showing children
    item_spacing: Vec2,
    size: [Option<f32>; 2],
}

impl<'a> FlexInstance<'a> {
    fn row_ui(parent: &mut Ui, row: Option<&RowData>) -> Ui {
        let rect = row.map_or(parent.max_rect(), |row| row.rect.unwrap());

        parent.new_child(UiBuilder::new().max_rect(rect))
    }

    /// Get the direction of the flex container.
    pub fn direction(&self) -> FlexDirection {
        self.flex.direction
    }

    /// Is the flex container horizontal?
    pub fn is_horizontal(&self) -> bool {
        self.flex.direction == FlexDirection::Horizontal
    }

    /// Is the flex container vertical?
    pub fn is_vertical(&self) -> bool {
        self.flex.direction == FlexDirection::Vertical
    }

    /// Get the ui of the flex container (e.g. to read the style or access the context).
    pub fn ui(&self) -> &Ui {
        self.ui
    }

    /// Create a child ui to e.g. show a overlay over some component
    pub fn new_child(&mut self, ui_builder: UiBuilder) -> Ui {
        self.ui.new_child(ui_builder)
    }

    /// Show a flex container. This is split in a outer and inner [Ui]. The outer [Ui] will
    /// grow according to the flex layout, while the inner [Ui] will be centered / positioned
    /// based on the [FlexItem::align_self_content].
    /// Use the [FlexContainerUi] to show your content in the inner [Ui].
    #[allow(clippy::too_many_lines)] // TODO: Refactor this to be more readable
    pub fn add_container<R>(
        &mut self,
        item: FlexItem,
        container_ui: impl FnOnce(&mut Ui, FlexContainerUi) -> FlexContainerResponse<R>,
    ) -> InnerResponse<R> {
        let item = FlexItem {
            grow: item.grow.or(self.flex.default_item.grow),
            basis: item.basis.or(self.flex.default_item.basis),
            align_self: item.align_self.or(self.flex.default_item.align_self),
            align_content: item.align_content.or(self.flex.default_item.align_content),
            shrink: item.shrink,
        };

        let row = self.rows.get_mut(self.current_row);

        if let Some(row) = &row {
            if self.current_row_index == 0 {
                self.row_ui.add_space(row.extra_start_gap);
            } else {
                self.row_ui.add_space(row.extra_gap);
            }
            row.extra_start_gap
        } else {
            0.0
        };

        let res = self.row_ui.scope(|ui| {
            let res = if let Some(row) = row {
                let row_item_count = row.items.len();
                // TODO: Handle when this is not set (Why doesn't this fail?)
                let item_state = row.items.get_mut(self.current_row_index).unwrap();

                let extra_length = if item_state.config.grow.unwrap_or(0.0) > 0.0
                    && row.total_grow > 0.0
                {
                    f32::max(
                        row.extra_space * item_state.config.grow.unwrap_or(0.0) / row.total_grow,
                        0.0,
                    )
                } else {
                    0.0
                };

                let do_shrink = item_state.config.shrink && row.extra_space < 0.0;

                let parent_min_rect = ui.min_rect();

                let mut total_size = item_state.min_size_with_margin();
                total_size[self.direction] += extra_length;

                if do_shrink {
                    total_size[self.direction] += row.extra_space;
                    if total_size[self.direction] < 0.0 {
                        total_size[self.direction] = 0.0;
                    }
                }

                let available_size = ui.available_rect_before_wrap().size();

                // If everything is wrapped we will limit the items size to the containers available
                // size to prevent it from growing out of the container
                if self.flex.wrap && row_item_count == 1 {
                    total_size[self.direction] =
                        f32::min(total_size[self.direction], available_size[self.direction]);
                }

                let align = item.align_self.unwrap_or_default();

                let frame_align = match align {
                    FlexAlign::Start => Some(Align::Min),
                    FlexAlign::End => Some(Align::Max),
                    FlexAlign::Center => Some(Align::Center),
                    FlexAlign::Stretch => {
                        total_size[1 - self.direction] = row.cross_size_with_extra_space;
                        None
                    }
                };

                let mut max_rect = ui.max_rect();

                // TODO: Is this right?
                if total_size[self.direction] > max_rect.size()[self.direction] {
                    let mut size = max_rect.size();
                    size[self.direction] = total_size[self.direction];
                    max_rect = Rect::from_min_size(max_rect.min, size);
                }

                let frame_rect = match frame_align {
                    None => Rect::from_min_size(parent_min_rect.min, total_size),
                    Some(align) => {
                        let mut align2 = Align2::LEFT_TOP;
                        align2[1 - self.direction] = align;
                        align2.align_size_within_rect(total_size, max_rect)
                    }
                };

                // ui.ctx()
                //     .debug_painter()
                //     .debug_rect(frame_rect, egui::Color32::RED, "");

                let mut target_inner_size = item_state.inner_size;

                if do_shrink {
                    target_inner_size[self.direction] =
                        total_size[self.direction] - item_state.margin.sum()[self.direction];
                }

                let content_align = item.align_content.unwrap_or(Align2::CENTER_CENTER);

                let frame_without_margin = frame_rect - item_state.margin;

                let mut content_rect =
                    content_align.align_size_within_rect(target_inner_size, frame_without_margin);

                let max_content_size = self.max_item_size - item_state.margin.sum();

                // If we shrink, we have to limit the size.
                if !do_shrink {
                    // Because we want to allow the content to grow (e.g. in case the text gets longer),
                    // we set the content_rect's size to match the flex ui's available size.
                    content_rect.set_width(max_content_size.x);
                    content_rect.set_height(max_content_size.y);
                }
                // We only want to limit the content size in the main dir
                // TODO: Should there be an option to also limit it in the cross dir?
                // content_rect.max[1 - self.direction] = self.max_item_size[1 - self.direction];
                // frame_rect.set_width(self.ui.available_width());
                // frame_rect.set_height(self.ui.available_height());

                if let Some(basis) = item.basis {
                    let mut size = content_rect.size();
                    size[self.direction] = basis + extra_length;
                    content_rect = Rect::from_min_size(
                        content_rect.min,
                        size.min(self.ui.available_size() - item_state.margin.sum()),
                    );
                }

                let mut child_ui =
                    ui.new_child(UiBuilder::new().max_rect(frame_rect).layout(*ui.layout()));
                child_ui.spacing_mut().item_spacing = self.item_spacing;

                let res = container_ui(
                    &mut child_ui,
                    FlexContainerUi {
                        direction: self.direction,
                        content_rect,
                        frame_rect,
                        margin: item_state.margin,
                        max_item_size: max_content_size,
                        // If the available space grows we want to remeasure the widget, in case
                        // it's wrapped so it can un-wrap
                        remeasure_widget: item_state.remeasure_widget
                            || self.max_item_size[self.direction]
                                != self.last_max_item_size[self.direction],
                        last_inner_size: Some(item_state.inner_size),
                        target_inner_size,
                        item,
                    },
                );
                let (_, _r) = ui.allocate_space(child_ui.min_rect().size());

                let mut inner_size = res.child_rect.size();
                if do_shrink {
                    let this_frame = res.child_rect.size()[self.direction];
                    let last_frame = item_state.inner_size[self.direction];
                    let target_size_this_frame = target_inner_size[self.direction];
                    inner_size[self.direction] = if this_frame < target_size_this_frame - 10.0 {
                        // The content must have changed and is now significantly below a width where
                        // we shouldn't have to shrink anymore, so we return the new size
                        this_frame
                    } else {
                        // We are currently shrunken, so we have to return the old size
                        last_frame
                    }
                };

                (inner_size, res, row.items.len(), child_ui.min_rect())
            } else {
                ui.set_invisible();

                let rect = self.ui.available_rect_before_wrap();

                let res = container_ui(
                    ui,
                    FlexContainerUi {
                        direction: self.direction,
                        content_rect: rect,
                        frame_rect: rect,
                        margin: Margin::ZERO,
                        max_item_size: self.max_item_size,
                        remeasure_widget: false,
                        last_inner_size: None,
                        target_inner_size: rect.size(),
                        item,
                    },
                );

                (res.child_rect.size(), res, 0, self.ui.min_rect())
            };

            let (mut inner_size, res, row_len, outer_rect) = res;

            // TODO: This calculates the top left margin, bottom right doesn't work as expected
            // let margin_bottom_right = outer_rect.max - res.container_min_rect.max;
            let margin_bottom_right = res.container_min_rect.min - outer_rect.min;
            let margin = round_margin(Margin {
                top: res.margin_top_left.y,
                left: res.margin_top_left.x,
                bottom: margin_bottom_right.y,
                right: margin_bottom_right.x,
            });
            if let Some(basis) = item.basis {
                inner_size[self.direction] = basis;
            }

            let item = ItemState {
                margin,
                inner_size: round_vec2(inner_size),
                id: ui.id(),
                inner_min_size: round_vec2(Vec2::max(res.min_size, inner_size)),
                config: item,
                remeasure_widget: res.remeasure_widget,
            };

            (res.inner, item, row_len)
        });
        let (inner, item, row_len) = res.inner;

        let is_last_item = self.current_row_index + 1 >= row_len;
        // TODO: Find a better way to do this, maybe just set the row ui rect to it's max rect?
        // if is_last_item
        //     && self.flex.justify != FlexJustify::Start
        //     && self.flex.justify != FlexJustify::End
        //     && self.flex.justify != FlexJustify::SpaceBetween
        // {
        //     let spacing = mem::take(&mut self.row_ui.spacing_mut().item_spacing);
        //     if self.direction == 0 {
        //         self.row_ui.add_space(self.row_ui.available_width());
        //     } else {
        //         self.row_ui.add_space(self.row_ui.available_height());
        //     }
        //     self.row_ui.spacing_mut().item_spacing = spacing;
        // }
        if let Some(row) = self.rows.get_mut(self.current_row) {
            let mut final_rect = self.row_ui.min_rect();
            if self.size[self.direction].is_some() {
                final_rect = final_rect.union(self.row_ui.max_rect());
            }
            row.final_rect = Some(final_rect);
        }

        self.state.items.push(item);

        self.current_row_index += 1;
        if is_last_item {
            self.current_row += 1;
            self.current_row_index = 0;
            self.row_ui = FlexInstance::row_ui(self.ui, self.rows.get(self.current_row));
        }

        InnerResponse::new(inner, res.response)
    }

    /// Add a simple item to the flex container.
    /// It will be positioned based on [FlexItem::align_self_content].
    #[deprecated = "Use `add_ui` instead"]
    pub fn add_simple<R>(
        &mut self,
        item: FlexItem,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| container.content(ui, content))
    }

    /// Add a child ui to the flex container.
    /// It will be positioned based on [FlexItem::align_self_content].
    pub fn add_ui<R>(
        &mut self,
        item: FlexItem,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| container.content(ui, content))
    }

    /// Add a [`FlexWidget`] to the flex container.
    /// [`FlexWidget`] is implemented for all default egui widgets.
    /// If you use a custom third party widget you can use [`Self::add_widget`] instead.
    pub fn add<W: FlexWidget>(&mut self, item: FlexItem, widget: W) -> InnerResponse<W::Response> {
        self.add_container(item, |ui, container| widget.flex_ui(ui, container))
    }

    /// Add a [`egui::Widget`] to the flex container.
    /// The default egui widgets implement [`FlexWidget`] Aso you can just use [`Self::add`] instead.
    /// If the widget reports it's intrinsic size via the [`egui::Response`] it will be able to
    /// grow it's frame according to the flex layout.
    pub fn add_widget<W: Widget>(&mut self, item: FlexItem, widget: W) -> InnerResponse<Response> {
        self.add_container(item, |ui, container| container.content_widget(ui, widget))
    }

    /// Add some content with a frame. The frame will be stretched according to the flex layout.
    /// The content will be centered / positioned based on [FlexItem::align_self_content].
    #[deprecated = "Use `add_ui_frame` instead"]
    pub fn add_frame<R>(
        &mut self,
        item: FlexItem,
        frame: Frame,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| {
            frame.show(ui, |ui| container.content(ui, content)).inner
        })
    }

    /// Add some content with a frame. The frame will be stretched according to the flex layout.
    /// The content will be centered / positioned based on [FlexItem::align_self_content].
    pub fn add_ui_frame<R>(
        &mut self,
        item: FlexItem,
        frame: Frame,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| {
            frame.show(ui, |ui| container.content(ui, content)).inner
        })
    }

    /// Add a nested flex container. Currently this doesn't correctly support wrapping the content
    /// in the nested container (once the content wraps, you will get weird results).
    #[track_caller]
    pub fn add_flex<R>(
        &mut self,
        item: FlexItem,
        flex: Flex,
        content: impl FnOnce(&mut FlexInstance) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| {
            container.content_flex(ui, flex, content)
        })
    }

    /// Add a nested flex container with a frame.
    /// See [`Self::add_flex`] for limitations.
    #[track_caller]
    pub fn add_flex_frame<R>(
        &mut self,
        item: FlexItem,
        mut flex: Flex,
        frame: Frame,
        content: impl FnOnce(&mut FlexInstance) -> R,
    ) -> InnerResponse<R> {
        // TODO: Is this correct behavior?
        if item
            .grow
            .or(self.flex.default_item.grow)
            .is_some_and(|g| g > 0.0)
            && self.flex.direction != flex.direction
        {
            flex.align_content = FlexAlignContent::Stretch;
        }

        self.add_container(item, |ui, container| {
            frame
                .show(ui, |ui| container.content_flex(ui, flex, content))
                .inner
        })
    }

    /// Adds an empty item with flex-grow 1.0.
    pub fn grow(&mut self) -> Response {
        self.add_ui(FlexItem::new().grow(1.0), |_| {}).response
    }
}

/// Helper to show the inner content of a container.
pub struct FlexContainerUi {
    direction: usize,
    content_rect: Rect,
    frame_rect: Rect,
    margin: Margin,
    max_item_size: Vec2,
    remeasure_widget: bool,
    last_inner_size: Option<Vec2>,
    target_inner_size: Vec2,
    item: FlexItem,
}

/// The response of the inner content of a container, should be passed as a return value from the
/// closure.
pub struct FlexContainerResponse<T> {
    child_rect: Rect,
    /// The response from the inner content.
    pub inner: T,
    margin_top_left: Vec2,
    max_size: Vec2,
    min_size: Vec2,
    container_min_rect: Rect,
    remeasure_widget: bool,
}

impl<T> FlexContainerResponse<T> {
    /// Map the inner value of the response.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> FlexContainerResponse<U> {
        FlexContainerResponse {
            child_rect: self.child_rect,
            inner: f(self.inner),
            margin_top_left: self.margin_top_left,
            max_size: self.max_size,
            min_size: self.min_size,
            container_min_rect: self.container_min_rect,
            remeasure_widget: self.remeasure_widget,
        }
    }
}

impl FlexContainerUi {
    /// Add the inner content of the container.
    pub fn content<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> FlexContainerResponse<R> {
        let Self {
            content_rect,
            frame_rect,
            margin,
            ..
        } = self;

        let margin_top_left = ui.min_rect().min - frame_rect.min;

        // TODO: Which one is correct?
        let child_rect = content_rect;
        // let child_rect = content_rect.intersect(ui.max_rect());

        let min_size = ui.min_size();

        let mut child = ui.new_child(UiBuilder::new().max_rect(child_rect));

        let r = content(&mut child);

        let child_min_rect = child.min_rect();

        ui.allocate_exact_size(
            Vec2::max(frame_rect.size() - margin.sum(), Vec2::ZERO),
            Sense::hover(),
        );

        let container_min_rect = ui.min_rect();

        FlexContainerResponse {
            inner: r,
            child_rect: child_min_rect,
            max_size: ui.available_size(),
            min_size,
            margin_top_left,
            container_min_rect,
            remeasure_widget: false,
        }
    }

    /// Add a nested flex container.
    #[track_caller]
    pub fn content_flex<R>(
        self,
        ui: &mut Ui,
        mut flex: Flex,
        content: impl FnOnce(&mut FlexInstance) -> R,
    ) -> FlexContainerResponse<R> {
        let Self {
            frame_rect,
            margin,
            mut max_item_size,
            remeasure_widget: _,
            last_inner_size: _,
            item,
            target_inner_size,
            ..
        } = self;

        // We will assume that the margin is symmetrical
        let margin_top_left = ui.min_rect().min - frame_rect.min;
        let container_min_size = ui.min_size();

        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());

        // We set wrap to false since we currently don't support wrapping in nested flexes
        flex = flex.wrap(false);

        // Make sure the container actually grows if grow is set.
        if self.item.grow.is_some_and(|g| g > 0.0) {
            #[allow(clippy::collapsible_else_if)]
            if self.direction == 0 {
                if flex.width.is_none() {
                    flex = flex.w_full();
                }
            } else {
                if flex.height.is_none() {
                    flex = flex.h_full();
                }
            }
        }

        let mut target_size = target_inner_size;
        // Limit the max item size if a basis is set. This could be done prettier but works for now.
        if item.basis.is_some() {
            max_item_size[self.direction] =
                f32::min(max_item_size[self.direction], target_size[self.direction]);
        }

        target_size = Vec2::min(target_size, max_item_size);

        let (min_size, res) =
            flex.show_inside(ui, Some(target_size), Some(max_item_size), |instance| {
                content(instance)
            });

        let container_min_rect = ui.min_rect();

        FlexContainerResponse {
            inner: res.inner,
            child_rect: Rect::from_min_size(frame_rect.min, min_size),
            max_size: ui.available_size(),
            min_size: container_min_size,
            margin_top_left,
            container_min_rect,
            remeasure_widget: false,
        }
    }

    /// Add a widget to the container.
    pub fn content_widget(
        self,
        ui: &mut Ui,
        widget: impl Widget,
    ) -> FlexContainerResponse<Response> {
        let margin_top_left = ui.min_rect().min - self.frame_rect.min;
        let min_size = ui.min_size();

        let id_salt = ui.id().with("flex_widget");
        let mut builder = UiBuilder::new()
            .id_salt(id_salt)
            .layout(Layout::centered_and_justified(Direction::TopDown));
        if self.remeasure_widget {
            ui.ctx().request_discard("Flex item remeasure");
            builder = builder.max_rect(self.content_rect).invisible();
        } else {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());
        };
        let response = ui.scope_builder(builder, |ui| widget.ui(ui)).inner;

        let intrinsic_size = response.intrinsic_size.map_or(
            Vec2::new(ui.spacing().interact_size.x, ui.spacing().interact_size.y),
            // Add some vertical space to prevent edge cases where text might wrap
            |s| s + Vec2::X * 1.0,
        );

        // If the size changed in the cross direction the widget might have grown in the main direction
        // and wrapped, we need to remeasure the widget (draw it once with full available size)
        let remeasure_widget = self.last_inner_size.is_some_and(|last_size| {
            round(last_size[1 - self.direction]) != round(intrinsic_size[1 - self.direction])
        }) && !self.remeasure_widget;

        if remeasure_widget {
            ui.ctx().request_repaint();
            ui.ctx().request_discard("Triggering flex item remeasure");
        }

        FlexContainerResponse {
            child_rect: Rect::from_min_size(self.frame_rect.min, intrinsic_size),
            inner: response,
            max_size: ui.available_size(),
            min_size,
            margin_top_left,
            container_min_rect: ui.min_rect(),
            remeasure_widget,
        }
    }
}

/// Round a float to 5 decimal places.
fn round(i: f32) -> f32 {
    const PRECISION: f32 = 1e3;
    let i = (i * PRECISION).round() / PRECISION;
    // I've seen this flip from 0.0 to -0.0 in a discard-loop
    if i == -0.0 {
        0.0
    } else {
        i
    }
}

fn round_vec2(v: Vec2) -> Vec2 {
    Vec2::new(round(v.x), round(v.y))
}

fn round_pos2(p: Pos2) -> Pos2 {
    Pos2::new(round(p.x), round(p.y))
}

fn round_margin(margin: Margin) -> Margin {
    Margin {
        top: round(margin.top),
        left: round(margin.left),
        bottom: round(margin.bottom),
        right: round(margin.right),
    }
}

fn round_rect(rect: Rect) -> Rect {
    Rect {
        min: round_pos2(rect.min),
        max: round_pos2(rect.max),
    }
}
