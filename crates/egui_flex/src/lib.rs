#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod flex_widget;

pub use crate::flex_widget::FlexWidget;
use egui::emath::{GuiRounding, TSTransform};
use egui::{
    Align, Align2, Direction, Frame, Id, InnerResponse, Layout, Margin, Pos2, Rect, Response,
    Sense, Ui, UiBuilder, Vec2, Widget,
};
use std::fmt::Debug;
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

/// A size value, either in points or as a percentage of the available space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// Size in points (pixels).
    Points(f32),
    /// Size as a percentage of the available space.
    Percent(f32),
}

impl From<f32> for Size {
    fn from(p: f32) -> Self {
        Size::Points(p)
    }
}

impl Size {
    /// Get the size in points (pixels) based on the total available space.
    pub fn get(&self, total: f32) -> f32 {
        match self {
            Size::Points(p) => *p,
            Size::Percent(p) => total * *p,
        }
    }
}

/// A flex container.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Flex {
    id_salt: Option<Id>,
    direction: FlexDirection,
    justify: FlexJustify,
    align_content: FlexAlignContent,
    gap: Option<Vec2>,
    default_item: FlexItemInner,
    wrap: bool,
    width: Option<Size>,
    height: Option<Size>,
}

type FrameBuilder<'a> = Box<dyn FnOnce(&Ui, &Response) -> (Frame, TSTransform) + 'a>;

/// Configuration for a flex item.
#[derive(Default)]
pub struct FlexItem<'a> {
    frame_builder: Option<FrameBuilder<'a>>,
    inner: FlexItemInner,
}

impl FlexItem<'_> {
    fn build_into_inner(self, ui: &Ui, response: &Response) -> FlexItemInner {
        let FlexItem {
            mut inner,
            frame_builder,
        } = self;
        if let Some(builder) = frame_builder {
            let (frame, transform) = builder(ui, response);
            inner.frame = Some(frame);
            inner.transform = Some(transform);
        }
        inner
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct FlexItemInner {
    grow: Option<f32>,
    basis: Option<f32>,
    align_self: Option<FlexAlign>,
    align_content: Option<Align2>,
    shrink: bool,
    frame: Option<Frame>,
    transform: Option<TSTransform>,
    content_id: Option<Id>,
    sense: Option<Sense>,
    min_size: [Option<f32>; 2],
}

/// Only the things that are relevant on the next frame
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct FlexItemState {
    grow: Option<f32>,
    basis: Option<f32>,
    shrink: bool,
    margin: Margin,
    content_id: Option<Id>,
}

impl FlexItemInner {
    fn or(self, b: FlexItemInner) -> FlexItemInner {
        FlexItemInner {
            grow: self.grow.or(b.grow),
            basis: self.basis.or(b.basis),
            align_self: self.align_self.or(b.align_self),
            align_content: self.align_content.or(b.align_content),
            shrink: self.shrink || b.shrink,
            frame: self.frame.or(b.frame),
            transform: self.transform.or(b.transform),
            content_id: self.content_id.or(b.content_id),
            sense: self.sense.or(b.sense),
            min_size: [
                self.min_size[0].or(b.min_size[0]),
                self.min_size[1].or(b.min_size[1]),
            ],
        }
    }

    fn into_state(self) -> FlexItemState {
        FlexItemState {
            grow: self.grow,
            basis: self.basis,
            shrink: self.shrink,
            margin: self.frame.map_or(Margin::ZERO, |f| f.total_margin().into()),
            content_id: self.content_id,
        }
    }
}

/// Create a new flex item. Shorthand for [`FlexItem::default`].
pub fn item() -> FlexItem<'static> {
    FlexItem::default()
}

impl<'a> FlexItem<'a> {
    /// Create a new flex item. You can also use the [`item`] function.
    pub fn new() -> Self {
        Self::default()
    }

    /// How much should this item grow compared to the other items.
    ///
    /// By default items don't grow.
    pub fn grow(mut self, grow: f32) -> Self {
        self.inner.grow = Some(grow);
        self
    }

    /// Set the default size of the item, before it grows.
    /// If this is not set, the items "intrinsic size" will be used.
    pub fn basis(mut self, basis: f32) -> Self {
        self.inner.basis = Some(basis);
        self
    }

    /// How do we align the item in the cross axis?
    ///
    /// Default is `stretch`.
    pub fn align_self(mut self, align_self: FlexAlign) -> Self {
        self.inner.align_self = Some(align_self);
        self
    }

    /// If `align_self` is stretch, how do we align the content?
    ///
    /// Default is `center`.
    pub fn align_self_content(mut self, align_self_content: Align2) -> Self {
        self.inner.align_content = Some(align_self_content);
        self
    }

    /// Shrink this item if there isn't enough space.
    ///
    /// Note: You may only ever set this on a single item in a flex container.
    pub fn shrink(mut self) -> Self {
        self.inner.shrink = true;
        self
    }

    /// Set the frame of the item.
    pub fn frame(mut self, frame: Frame) -> Self {
        self.inner.frame = Some(frame);
        self
    }

    /// Set the visual transform of the item.
    pub fn transform(mut self, transform: TSTransform) -> Self {
        self.inner.transform = Some(transform);
        self
    }

    /// Set the frame of the item using a builder function.
    pub fn frame_builder(
        mut self,
        frame_builder: impl FnOnce(&Ui, &Response) -> (Frame, TSTransform) + 'a,
    ) -> Self {
        self.frame_builder = Some(Box::new(frame_builder));
        self
    }

    /// Egui flex can't always know when the content size of a widget changes (e.g. when a Label
    /// or Button is truncated). If the content id changes, this will force a remeasure of the
    /// widget.
    pub fn content_id(mut self, content_id: Id) -> Self {
        self.inner.content_id = Some(content_id);
        self
    }

    /// Set a sense for the FlexItem. The response will be passed to the FrameBuilder closure.
    pub fn sense(mut self, sense: Sense) -> Self {
        self.inner.sense = Some(sense);
        self
    }

    /// Set the minimum outer (including Frame margin) size
    /// of the item in points (pixels).
    pub fn min_size(mut self, min_size: impl Into<Vec2>) -> Self {
        let min_size = min_size.into();
        self.inner.min_size = [Some(min_size.x), Some(min_size.y)];
        self
    }

    /// Set the minimum outer (including Frame margin) width
    /// of the item in points (pixels).
    pub fn min_width(mut self, min_width: impl Into<Option<f32>>) -> Self {
        self.inner.min_size[0] = min_width.into();
        self
    }

    /// Set the minimum outer (including Frame margin) height
    /// of the item in points (pixels).
    pub fn min_height(mut self, min_height: impl Into<Option<f32>>) -> Self {
        self.inner.min_size[1] = min_height.into();
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
    pub fn grow_items(mut self, grow: impl Into<Option<f32>>) -> Self {
        self.default_item.grow = grow.into();
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
                .max_rect(ui.available_rect_before_wrap().round_ui()),
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
                    width.map(|w| w.get(size_origin.x).round_ui()),
                    height.map(|h| h.get(size_origin.y).round_ui()),
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

                let max_item_size = max_item_size.unwrap_or(available_size).round_ui();

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

                min_size = min_size.min(max_item_size);

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
                    basis + item.config.margin.sum()[direction]
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
        let mut _extra_cross_gap_end = 0.0; // TODO: How to handle extra end space?
        let mut extra_cross_space_per_row = 0.0;

        if !self.wrap {
            self.align_content = FlexAlignContent::Stretch;
        }
        match self.align_content {
            FlexAlignContent::Start => {
                _extra_cross_gap_end = extra_cross_space;
            }
            FlexAlignContent::Stretch => {
                extra_cross_space_per_row = extra_cross_space / rows.len() as f32;
            }
            FlexAlignContent::End => {
                extra_cross_gap_start = extra_cross_space;
            }
            FlexAlignContent::Center => {
                extra_cross_gap_start = extra_cross_space / 2.0;
                _extra_cross_gap_end = extra_cross_space / 2.0;
            }
            FlexAlignContent::SpaceBetween => {
                extra_cross_gap = extra_cross_space / (rows.len() as f32 - 1.0);
            }
            FlexAlignContent::SpaceAround => {
                extra_cross_gap = extra_cross_space / rows.len() as f32;
                extra_cross_gap_start = extra_cross_gap / 2.0;
                _extra_cross_gap_end = extra_cross_gap / 2.0;
            }
        };

        let mut row_position = min_position;

        row_position[cross_direction] += extra_cross_gap_start;

        let row_count = rows.len();
        for (_idx, row) in &mut rows.iter_mut().enumerate() {
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
            if size[direction].is_some() || row_count > 1 || diff < 0.0 {
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
                row.extra_gap = f32::max(row.extra_gap.round_ui(), 0.0);
                row.extra_start_gap = f32::max(row.extra_start_gap.round_ui(), 0.0);
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

    /// Show this flex in another Flex. See also [FlexInstance::add_flex].
    #[track_caller]
    pub fn show_in<R>(
        self,
        flex: &mut FlexInstance,
        item: FlexItem,
        f: impl FnOnce(&mut FlexInstance) -> R,
    ) -> InnerResponse<R> {
        flex.add_flex(item, self, f)
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
    config: FlexItemState,
    inner_size: Vec2,
    inner_min_size: Vec2,
    remeasure_widget: bool,
}

impl ItemState {
    fn min_size_with_margin(&self) -> Vec2 {
        self.inner_min_size + self.config.margin.sum()
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

impl FlexInstance<'_> {
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
        &self.row_ui
    }

    /// Access the underlying [`egui::Painter`].
    pub fn painter(&self) -> &egui::Painter {
        self.row_ui.painter()
    }

    /// Access the underlying [`egui::Output`].
    pub fn visuals(&self) -> &egui::style::Visuals {
        self.row_ui.visuals()
    }

    /// Access the underlying [`egui::style::Visuals`].
    pub fn visuals_mut(&mut self) -> &mut egui::style::Visuals {
        self.row_ui.visuals_mut()
    }

    /// Access the underlying [`egui::style::Style`].
    pub fn style(&self) -> &egui::style::Style {
        self.row_ui.style()
    }

    /// Access the underlying [`egui::style::Style`] mutably.
    pub fn style_mut(&mut self) -> &mut egui::style::Style {
        self.row_ui.style_mut()
    }

    /// Access the underlying [`egui::Spacing`].
    pub fn spacing(&self) -> &egui::Spacing {
        self.row_ui.spacing()
    }

    /// Create a child ui to e.g. show a overlay over some component
    pub fn new_child(&mut self, ui_builder: UiBuilder) -> Ui {
        self.ui.new_child(ui_builder)
    }

    #[allow(clippy::too_many_lines)] // TODO: Refactor this to be more readable
    fn add_container<R>(&mut self, mut item: FlexItem, content: ContentFn<R>) -> InnerResponse<R> {
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

        item.inner = item.inner.or(self.flex.default_item);

        let res = self.row_ui.scope_builder(
            UiBuilder::new().sense(item.inner.sense.unwrap_or(Sense::hover())),
            |ui| {
                let item = item.build_into_inner(ui, &ui.response());

                let basis = item.basis;

                let frame = item.frame.unwrap_or_default();
                let transform = item.transform.unwrap_or_default();
                let margin = frame.inner_margin + frame.outer_margin;

                let res = if let Some(row) = row {
                    let row_item_count = row.items.len();
                    // TODO: Handle when this is not set (Why doesn't this fail?)
                    let item_state = row.items.get_mut(self.current_row_index).unwrap();

                    let extra_length =
                        if item_state.config.grow.unwrap_or(0.0) > 0.0 && row.total_grow > 0.0 {
                            f32::max(
                                row.extra_space * item_state.config.grow.unwrap_or(0.0)
                                    / row.total_grow,
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
                            total_size[self.direction] - margin.sum()[self.direction];
                    }

                    let content_align = item.align_content.unwrap_or(Align2::CENTER_CENTER);

                    let frame_without_margin = frame_rect - margin;

                    let mut content_rect = content_align
                        .align_size_within_rect(target_inner_size, frame_without_margin);

                    if let Some(basis) = item.basis {
                        let mut size = content_rect.size();
                        size[self.direction] = basis + extra_length;
                        content_rect = Rect::from_center_size(
                            content_rect.center(),
                            size.min(self.ui.available_size() - item_state.config.margin.sum()),
                        );
                    }

                    // ui.ctx()
                    //     .debug_painter()
                    //     .debug_rect(content_rect, egui::Color32::RED, "");

                    let max_content_size = self.max_item_size - margin.sum();

                    // If we shrink, we have to limit the size, otherwise we let it grow to max_content_size
                    if !do_shrink && item.basis.is_none() {
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

                    // ui.ctx()
                    //     .debug_painter()
                    //     .debug_rect(frame_rect, egui::Color32::GREEN, "");

                    let mut child_ui =
                        ui.new_child(UiBuilder::new().max_rect(frame_rect).layout(*ui.layout()));
                    child_ui.spacing_mut().item_spacing = self.item_spacing;

                    let res = child_ui
                        .with_visual_transform(transform, |ui| {
                            frame
                                .show(ui, |ui| {
                                    content(
                                        ui,
                                        FlexContainerUi {
                                            direction: self.direction,
                                            content_rect,
                                            frame_rect,
                                            margin: item_state.config.margin,
                                            max_item_size: max_content_size,
                                            // If the available space grows we want to remeasure the widget, in case
                                            // it's wrapped so it can un-wrap
                                            remeasure_widget: item_state.remeasure_widget
                                                || self.max_item_size[self.direction]
                                                    != self.last_max_item_size[self.direction]
                                                || item.content_id != item_state.config.content_id,
                                            last_inner_size: Some(item_state.inner_size),
                                            target_inner_size,
                                            item,
                                        },
                                    )
                                })
                                .inner
                        })
                        .inner;
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

                    (inner_size, res, row.items.len())
                } else {
                    ui.set_invisible();

                    let rect = self.ui.available_rect_before_wrap();

                    let res = content(
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

                    (res.child_rect.size(), res, 0)
                };

                let (mut inner_size, res, row_len) = res;

                if let Some(basis) = basis {
                    inner_size[self.direction] = basis;
                }

                let item = ItemState {
                    inner_size: inner_size.round_ui(),
                    id: ui.id(),
                    inner_min_size: Vec2::max(
                        Vec2::new(
                            item.min_size[0].unwrap_or_default(),
                            item.min_size[1].unwrap_or_default(),
                        ) - frame.total_margin().sum(),
                        inner_size,
                    )
                    .round_ui(),
                    config: item.into_state(),
                    remeasure_widget: res.remeasure_widget,
                };

                (res.inner, item, row_len)
            },
        );
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

    /// Add a child ui to the flex container.
    /// It will be positioned based on [FlexItem::align_self_content].
    pub fn add_ui<R>(
        &mut self,
        item: FlexItem,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_container(
            item,
            Box::new(|ui, container| container.content(ui, content)),
        )
    }

    /// Add a [`FlexWidget`] to the flex container.
    /// [`FlexWidget`] is implemented for all default egui widgets.
    /// If you use a custom third party widget you can use [`Self::add_widget`] instead.
    pub fn add<W: FlexWidget>(&mut self, item: FlexItem, widget: W) -> W::Response {
        widget.flex_ui(item, self)
    }

    /// Add a [`Widget`] to the flex container.
    /// The default egui widgets implement [`FlexWidget`] Also you can just use [`Self::add`] instead.
    /// If the widget reports it's intrinsic size via the [`Response`] it will be able to
    /// grow it's frame according to the flex layout.
    pub fn add_widget<W: Widget>(&mut self, item: FlexItem, widget: W) -> InnerResponse<Response> {
        self.add_container(
            item,
            Box::new(|ui, container| container.content_widget(ui, widget)),
        )
    }

    /// Add a nested flex container. Currently this doesn't correctly support wrapping the content
    /// in the nested container (once the content wraps, you will get weird results).
    #[track_caller]
    pub fn add_flex<R>(
        &mut self,
        item: FlexItem,
        mut flex: Flex,
        content: impl FnOnce(&mut FlexInstance) -> R,
    ) -> InnerResponse<R> {
        // TODO: Is this correct behavior?
        if item
            .inner
            .grow
            .or(self.flex.default_item.grow)
            .is_some_and(|g| g > 0.0)
            && self.flex.direction != flex.direction
        {
            flex.align_content = FlexAlignContent::Stretch;
        }

        self.add_container(
            item,
            Box::new(|ui, container| container.content_flex(ui, flex, content)),
        )
    }

    /// Adds an empty item with flex-grow 1.0.
    pub fn grow(&mut self) -> Response {
        self.add_ui(FlexItem::new().grow(1.0), |_| {}).response
    }
}

type ContentFn<'a, R> = Box<dyn FnOnce(&mut Ui, FlexContainerUi) -> FlexContainerResponse<R> + 'a>;

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
    item: FlexItemInner,
}

/// The response of the inner content of a container, should be passed as a return value from the
/// closure.
pub struct FlexContainerResponse<T> {
    child_rect: Rect,
    /// The response from the inner content.
    pub inner: T,
    max_size: Vec2,
    remeasure_widget: bool,
}

impl<T> FlexContainerResponse<T> {
    /// Map the inner value of the response.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> FlexContainerResponse<U> {
        FlexContainerResponse {
            child_rect: self.child_rect,
            inner: f(self.inner),
            max_size: self.max_size,
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

        // TODO: Which one is correct?
        let child_rect = content_rect;
        // let child_rect = content_rect.intersect(ui.max_rect());

        let mut child = ui.new_child(UiBuilder::new().max_rect(child_rect));

        let r = content(&mut child);

        let child_min_rect = child.min_rect();

        ui.allocate_exact_size(
            Vec2::max(frame_rect.size() - margin.sum(), Vec2::ZERO),
            Sense::hover(),
        );

        FlexContainerResponse {
            inner: r,
            child_rect: child_min_rect,
            max_size: ui.available_size(),
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
            margin: _,
            mut max_item_size,
            remeasure_widget: _,
            last_inner_size: _,
            item,
            target_inner_size,
            ..
        } = self;

        ui.set_width(ui.available_width());
        ui.set_height(ui.available_height());

        // We set wrap to false since we currently don't support wrapping in nested flexes
        flex = flex.wrap(false);

        // Make sure the container actually grows if grow is set.
        if item.grow.is_some_and(|g| g > 0.0) {
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

        FlexContainerResponse {
            inner: res.inner,
            child_rect: Rect::from_min_size(frame_rect.min, min_size),
            max_size: ui.available_size(),
            remeasure_widget: false,
        }
    }

    /// Add a widget to the container.
    pub fn content_widget(
        self,
        ui: &mut Ui,
        widget: impl Widget,
    ) -> FlexContainerResponse<Response> {
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
            // Add some horizontal space to prevent edge cases where text might wrap
            |s| s + Vec2::X * 1.0,
        );

        // If the size changed in the cross direction the widget might have grown in the main direction
        // and wrapped, we need to remeasure the widget (draw it once with full available size)
        let remeasure_widget = self.last_inner_size.is_some_and(|last_size| {
            last_size[1 - self.direction].round_ui()
                != intrinsic_size[1 - self.direction].round_ui()
        }) && !self.remeasure_widget;

        if remeasure_widget {
            ui.ctx().request_repaint();
            ui.ctx().request_discard("Triggering flex item remeasure");
        }

        FlexContainerResponse {
            child_rect: Rect::from_min_size(self.frame_rect.min, intrinsic_size),
            inner: response,
            max_size: ui.available_size(),
            remeasure_widget,
        }
    }
}
