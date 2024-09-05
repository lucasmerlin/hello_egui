pub mod flex_button;
pub mod flex_widget;

use crate::flex_widget::FlexWidget;
use egui::{
    Align, Align2, Direction, Frame, Id, InnerResponse, Layout, Margin, Rect, Response, Sense, Ui,
    Vec2, Widget,
};
use std::mem;

#[derive(Debug, Clone, Copy, Default)]
pub enum FlexDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FlexJustify {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FlexAlign {
    Start,
    End,
    Center,
    #[default]
    Stretch,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FlexAlignContent {
    #[default]
    Normal,
    Start,
    End,
    Center,
    Stretch,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Default)]
pub struct Flex {
    direction: FlexDirection,
    justify: FlexJustify,
    align_content: FlexAlignContent,
    gap: Option<Vec2>,
    default_item: FlexItem,
}

#[derive(Debug, Clone, Default)]
pub struct FlexItem {
    grow: Option<f32>,
    basis: Option<f32>,
    align_self: Option<FlexAlign>,
    align_content: Option<Align2>,
}

impl FlexItem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grow(mut self, grow: f32) -> Self {
        self.grow = Some(grow);
        self
    }

    pub fn basis(mut self, basis: f32) -> Self {
        self.basis = Some(basis);
        self
    }

    pub fn align_self(mut self, align_self: FlexAlign) -> Self {
        self.align_self = Some(align_self);
        self
    }

    /// If align_self is stretch, how do we align the content?
    pub fn align_self_content(mut self, align_self_content: Align2) -> Self {
        self.align_content = Some(align_self_content);
        self
    }
}

impl Flex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn horizontal() -> Self {
        Self::default().direction(FlexDirection::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::default().direction(FlexDirection::Vertical)
    }

    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn justify(mut self, justify: FlexJustify) -> Self {
        self.justify = justify;
        self
    }

    pub fn align_items(mut self, align_items: FlexAlign) -> Self {
        self.default_item.align_self = Some(align_items);
        self
    }

    pub fn align_item_content(mut self, align_item_content: Align2) -> Self {
        self.default_item.align_content = Some(align_item_content);
        self
    }

    pub fn align_content(mut self, align_content: FlexAlignContent) -> Self {
        self.align_content = align_content;
        self
    }

    pub fn grow_items(mut self) -> Self {
        self.default_item.grow = Some(1.0);
        self
    }

    pub fn gap(mut self, gap: Vec2) -> Self {
        self.gap = Some(gap);
        self
    }

    fn show_inside<R>(
        self,
        ui: &mut Ui,
        target_size: Option<Vec2>,
        max_item_size: Option<Vec2>,
        f: impl FnOnce(&mut FlexInstance) -> R,
    ) -> (Vec2, R) {
        let id = ui.auto_id_with("flex");
        let state: FlexState = ui
            .ctx()
            .memory(|mem| mem.data.get_temp(id).clone().unwrap_or_default());

        let layout = match self.direction {
            FlexDirection::Horizontal => Layout::left_to_right(Align::Min),
            FlexDirection::Vertical => Layout::top_down(Align::Min),
        };

        let r = ui
            .with_layout(layout, |ui| {
                let gap = self.gap.unwrap_or(ui.spacing_mut().item_spacing);
                let item_spacing = mem::replace(&mut ui.spacing_mut().item_spacing, gap);

                let available_size = target_size.unwrap_or(ui.available_size());
                let direction = if ui.layout().main_dir().is_horizontal() {
                    0
                } else {
                    1
                };
                let cross_direction = 1 - direction;

                let available_length = available_size[direction];
                let item_spacing_direction = ui.spacing().item_spacing[direction];

                let mut rows = vec![];
                let mut current_row = RowData::default();
                for item in &state.items {
                    let item_length = item
                        .config
                        .basis
                        .map(|basis| basis + item.margin.sum()[direction])
                        .unwrap_or(item.size_with_margin[direction]);

                    if item_length + item_spacing_direction + current_row.total_size
                        > available_length
                        && !current_row.items.is_empty()
                    {
                        rows.push(mem::take(&mut current_row));
                    }

                    current_row.total_size += item_length;
                    if !current_row.items.is_empty() {
                        current_row.total_size += item_spacing_direction;
                    }
                    current_row.total_grow += item.config.grow.unwrap_or(0.0);
                    current_row.items.push(item.clone());
                    if item.size_with_margin[cross_direction] > current_row.cross_size {
                        current_row.cross_size = item.size_with_margin[cross_direction];
                    }
                }

                if !current_row.items.is_empty() {
                    rows.push(current_row);
                }

                let available_cross_size = available_size[cross_direction];
                let total_row_cross_size = rows.iter().map(|row| row.cross_size).sum::<f32>();
                let extra_cross_space_per_row = match self.align_content {
                    FlexAlignContent::Normal => 0.0,
                    FlexAlignContent::Stretch => {
                        let extra_cross_space = f32::max(
                            available_cross_size
                                - total_row_cross_size
                                - (rows.len().max(1) - 1) as f32
                                    * ui.spacing().item_spacing[cross_direction],
                            0.0,
                        );
                        let extra_cross_space_per_row = extra_cross_space / rows.len() as f32;
                        extra_cross_space_per_row
                    }
                    _ => unimplemented!(),
                };

                let mut row_position = ui.min_rect().min;

                for (i, row) in rows.iter_mut().enumerate() {
                    let mut row_size = Vec2::ZERO;
                    row_size[direction] = available_length;
                    row_size[cross_direction] = row.cross_size + extra_cross_space_per_row;
                    row_size[cross_direction] =
                        f32::min(row_size[cross_direction], available_size[cross_direction]);

                    row.cross_size_with_extra_space = row_size[cross_direction];
                    row.rect = Some(Rect::from_min_size(row_position, row_size));

                    // ui.ctx().debug_painter().debug_rect(
                    //     row.rect.unwrap(),
                    //     egui::Color32::from_rgba_unmultiplied(255, 255, 0, 128),
                    //     format!("row {}", i),
                    // );

                    row_position[cross_direction] +=
                        row_size[cross_direction] + ui.spacing().item_spacing[cross_direction];

                    row.extra_space = available_length - row.total_size;
                }

                let mut instance = FlexInstance {
                    current_index: 0,
                    current_row: 0,
                    current_row_index: 0,
                    flex: &self,
                    state: FlexState::default(),
                    direction,
                    row_ui: FlexInstance::row_ui(ui, rows.first()),
                    ui,
                    rows,
                    max_item_size,
                    item_spacing,
                };

                let r = f(&mut instance);

                let mut min_size =
                    instance
                        .state
                        .items
                        .iter()
                        .fold(Vec2::ZERO, |mut current, item| {
                            current[direction] += item.size_with_margin[direction];
                            current[cross_direction] = f32::max(
                                current[cross_direction],
                                item.size_with_margin[cross_direction],
                            );
                            current
                        });
                min_size[direction] +=
                    item_spacing_direction * (instance.state.items.len() as f32 - 1.0);

                instance.ui.ctx().memory_mut(|mem| {
                    mem.data.insert_temp(id, instance.state);
                });

                instance.rows.iter().for_each(|row| {
                    if let Some(final_rect) = row.final_rect {
                        instance.ui.allocate_rect(final_rect, Sense::hover());
                    }
                });
                (min_size, r)
            })
            .inner;

        r
    }

    pub fn show<R>(self, ui: &mut Ui, f: impl FnOnce(&mut FlexInstance) -> R) -> R {
        self.show_inside(ui, None, None, f).1
    }
}

#[derive(Debug, Clone, Default)]
struct RowData {
    items: Vec<ItemState>,
    total_size: f32,
    total_grow: f32,
    extra_space: f32,
    cross_size: f32,
    cross_size_with_extra_space: f32,
    rect: Option<Rect>,
    final_rect: Option<Rect>,
}

#[derive(Debug, Clone)]
struct ItemState {
    id: Id,
    config: FlexItem,
    size_with_margin: Vec2,
    inner_size: Vec2,
    margin: Margin,
}

#[derive(Debug, Clone, Default)]
struct FlexState {
    items: Vec<ItemState>,
}

pub struct FlexInstance<'a> {
    flex: &'a Flex,
    current_index: usize,
    current_row: usize,
    current_row_index: usize,
    state: FlexState,
    ui: &'a mut Ui,
    rows: Vec<RowData>,
    direction: usize,
    row_ui: Ui,
    max_item_size: Option<Vec2>,
    // Original item spacing to store when showing children
    item_spacing: Vec2,
}

impl<'a> FlexInstance<'a> {
    fn row_ui(parent: &mut Ui, row: Option<&RowData>) -> Ui {
        let rect = row
            .map(|row| row.rect.unwrap())
            .unwrap_or(parent.max_rect());
        let mut child = parent.child_ui(rect, *parent.layout(), None);
        // child.set_width(child.available_width());
        // child.set_height(child.available_height());
        child
    }

    pub fn ui(&self) -> &Ui {
        self.ui
    }

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
        };

        let row = self.rows.get_mut(self.current_row);
        //
        // self.row_ui.ctx().debug_painter().debug_rect(
        //     self.row_ui.min_rect(),
        //     egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128),
        //     format!("row {}", self.current_row),
        // );

        let res = self.row_ui.scope(|ui| {
            let res = if let Some(row) = row {
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

                let parent_min_rect = ui.min_rect();

                let mut total_size = item_state.size_with_margin;
                if let Some(basis) = item.basis {
                    total_size[self.direction] = basis + item_state.margin.sum()[self.direction];
                }
                total_size[self.direction] += extra_length;

                let available_size = ui.available_rect_before_wrap().size();
                total_size[self.direction] =
                    f32::min(total_size[self.direction], available_size[self.direction]);
                total_size[1 - self.direction] = f32::min(
                    total_size[1 - self.direction],
                    available_size[1 - self.direction],
                );

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

                let mut frame_rect = match frame_align {
                    None => Rect::from_min_size(parent_min_rect.min, total_size),
                    Some(align) => {
                        let mut align2 = Align2::LEFT_TOP;
                        align2[1 - self.direction] = align;
                        align2.align_size_within_rect(total_size, parent_min_rect)
                    }
                };

                let mut inner_size = item_state.inner_size;
                if let Some(basis) = item.basis {
                    inner_size[self.direction] = basis + extra_length;
                }
                inner_size[self.direction] = f32::min(
                    inner_size[self.direction],
                    available_size[self.direction] - item_state.margin.sum()[self.direction],
                );

                let content_align = item.align_content.unwrap_or(Align2::CENTER_CENTER);

                let frame_without_margin = Rect {
                    min: frame_rect.min + item_state.margin.left_top(),
                    max: frame_rect.max - item_state.margin.right_bottom(),
                };

                let mut content_rect =
                    content_align.align_size_within_rect(inner_size, frame_without_margin);

                let max_content_size =
                    self.max_item_size.unwrap_or(ui.available_size()) - item_state.margin.sum();
                // Because we want to allow the content to grow (e.g. in case the text gets longer),
                // we set the content_rect's size to match the flex ui's available size.
                content_rect.set_width(max_content_size.x);
                content_rect.set_height(max_content_size.y);
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

                // ui.ctx().debug_painter().debug_rect(
                //     frame_rect,
                //     egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128),
                //     format!("frame_rect {}", self.current_index),
                // );
                //
                // if item.basis.is_some() {
                // ui.ctx().debug_painter().debug_rect(
                //     content_rect,
                //     egui::Color32::from_rgba_unmultiplied(0, 255, 0, 128),
                //     format!("{}", self.current_index),
                // );
                // }

                let mut child_ui = ui.child_ui(frame_rect, *ui.layout(), None);
                child_ui.spacing_mut().item_spacing = self.item_spacing;

                let res = container_ui(
                    &mut child_ui,
                    FlexContainerUi {
                        direction: self.direction,
                        extra_length,
                        basis: item.basis,
                        content_rect,
                        frame_rect,
                        margin: item_state.margin,
                        parent_min_rect,
                        max_item_size: max_content_size,
                    },
                );
                let (_, _r) = ui.allocate_space(child_ui.min_rect().size());
                // ui.ctx().debug_painter().debug_rect(
                //     ui.min_rect(),
                //     egui::Color32::from_rgba_unmultiplied(0, 0, 255, 128),
                //     format!("allocated {}", self.current_index),
                // );

                (res, row.items.len(), frame_rect)
            } else {
                ui.set_invisible();

                let rect = self.ui.available_rect_before_wrap();

                let res = container_ui(
                    ui,
                    FlexContainerUi {
                        direction: self.direction,
                        extra_length: 0.0,
                        basis: item.basis,
                        content_rect: rect,
                        frame_rect: rect,
                        margin: Margin::ZERO,
                        parent_min_rect: rect,
                        max_item_size: self.max_item_size.unwrap_or(ui.available_size()),
                    },
                );

                (res, 0, rect)
            };

            let (res, row_len, frame_rect) = res;

            let margin_bottom_right = res.container_min_rect.min - frame_rect.min;
            let margin = Margin {
                top: res.margin_top_left.y,
                left: res.margin_top_left.x,
                bottom: margin_bottom_right.y,
                right: margin_bottom_right.x,
            };

            let item = ItemState {
                margin,
                inner_size: res.child_rect.size(),
                id: ui.id(),
                size_with_margin: res.child_rect.size() + margin.sum(),
                config: item,
            };

            (res.inner, item, row_len)
        });

        if let Some(row) = self.rows.get_mut(self.current_row) {
            row.final_rect = Some(self.row_ui.min_rect());
        }

        let (inner, item, row_len) = res.inner;

        self.state.items.push(item);

        self.current_row_index += 1;
        if self.current_row_index >= row_len {
            self.current_row += 1;
            self.current_row_index = 0;
            self.row_ui = FlexInstance::row_ui(self.ui, self.rows.get(self.current_row));
        }

        InnerResponse::new(inner, res.response)
    }

    pub fn add_simple<R>(
        &mut self,
        item: FlexItem,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| container.content(ui, content))
    }

    pub fn add<W: FlexWidget>(&mut self, item: FlexItem, widget: W) -> InnerResponse<W::Response> {
        self.add_container(item, |ui, container| widget.ui(ui, container))
    }

    pub fn add_widget<W: Widget>(&mut self, item: FlexItem, widget: W) -> InnerResponse<Response> {
        self.add_simple(item, |ui| widget.ui(ui))
    }

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

    pub fn add_flex_frame<R>(
        &mut self,
        item: FlexItem,
        flex: Flex,
        frame: Frame,
        content: impl FnOnce(&mut FlexInstance) -> R,
    ) -> InnerResponse<R> {
        self.add_container(item, |ui, container| {
            frame
                .show(ui, |ui| container.content_flex(ui, flex, content))
                .inner
        })
    }
}

pub struct FlexContainerUi {
    direction: usize,
    basis: Option<f32>,
    extra_length: f32,
    content_rect: Rect,
    frame_rect: Rect,
    margin: Margin,
    parent_min_rect: Rect,
    max_item_size: Vec2,
}

pub struct FlexContainerResponse<T> {
    child_rect: Rect,
    inner: T,
    margin_top_left: Vec2,
    max_size: Vec2,
    container_min_rect: Rect,
}

impl<T> FlexContainerResponse<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> FlexContainerResponse<U> {
        FlexContainerResponse {
            child_rect: self.child_rect,
            inner: f(self.inner),
            margin_top_left: self.margin_top_left,
            max_size: self.max_size,
            container_min_rect: self.container_min_rect,
        }
    }
}

impl FlexContainerUi {
    pub fn content<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> FlexContainerResponse<R> {
        let Self {
            direction,
            extra_length,
            basis,
            content_rect,
            frame_rect,
            margin,
            parent_min_rect,
            max_item_size,
        } = self;

        // We will assume that the margin is symmetrical
        let margin_top_left = ui.min_rect().min - frame_rect.min;

        let mut child = ui.child_ui(content_rect, *ui.layout(), None);

        let r = content(&mut child);

        let child_min_rect = child.min_rect();

        // let mut extended_size = child_min_rect.size();
        // if let Some(basis) = basis {
        //     extended_size[direction] = basis;
        // }
        // extended_size[direction] += extra_length;
        //
        // ui.allocate_exact_size(extended_size, Sense::hover());

        ui.allocate_exact_size(
            Vec2::max(frame_rect.size() - margin.sum(), Vec2::ZERO),
            Sense::hover(),
        );

        let container_min_rect = ui.min_rect();

        FlexContainerResponse {
            inner: r,
            child_rect: child_min_rect,
            max_size: ui.available_size(),
            margin_top_left,
            container_min_rect,
        }
    }

    pub fn content_flex<R>(
        self,
        ui: &mut Ui,
        flex: Flex,
        content: impl FnOnce(&mut FlexInstance) -> R,
    ) -> FlexContainerResponse<R> {
        // ui.ctx().debug_painter().debug_rect(
        //     self.frame_rect,
        //     egui::Color32::from_rgba_unmultiplied(0, 0, 255, 128),
        //     format!("frame_rect"),
        // );
        let Self {
            direction,
            basis,
            extra_length,
            content_rect,
            frame_rect,
            margin,
            parent_min_rect,
            max_item_size,
        } = self;

        // We will assume that the margin is symmetrical
        let margin_top_left = ui.min_rect().min - frame_rect.min;

        let (min_size, res) = flex.show_inside(
            ui,
            Some(frame_rect.size() - margin.sum()),
            Some(max_item_size),
            |instance| {
                let res = content(instance);

                res
            },
        );

        let container_min_rect = ui.min_rect();

        FlexContainerResponse {
            inner: res,
            child_rect: Rect::from_min_size(frame_rect.min, min_size),
            max_size: ui.available_size(),
            margin_top_left,
            container_min_rect,
        }
    }
}
