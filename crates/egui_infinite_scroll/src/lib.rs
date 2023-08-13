use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::{Deref, DerefMut, Range, RangeInclusive};

use egui::{Id, Rect, Response, Ui, Vec2};
use egui_inbox::UiInbox;

pub trait InfiniteScrollItem {
    type Cursor: Clone + Send + Sync;

    fn visible(&mut self) {}

    fn hidden(&mut self) {}

    fn cursor(&self) -> Self::Cursor;
}

#[derive(Debug)]
enum LoadingState<T, Cursor> {
    Loaded(Vec<T>, Option<Cursor>),
    Loading,
    Idle,
    NoMoreItems,
    Error(String),
}

// #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(? Send))]
// #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
// pub trait InfiniteScrollLoader<T: InfiniteScrollItem + MaybeSend + MaybeSync>:
//     Clone + MaybeSend + MaybeSync
// {
//     async fn load_top(&self, _previous_item: Option<&T>) -> Option<Result<Vec<T>, String>> {
//         None
//     }
//     async fn load_bottom(&self, previous_item: Option<T::Cursor>) -> anyhow::Result<Vec<T>>;
// }

#[derive(Debug)]
struct ItemData<T> {
    item: T,
    rect: Option<Rect>,
}

#[derive(Debug)]
struct RowData {
    range: Range<usize>,
    rect: Rect,
}

impl<T> From<T> for ItemData<T> {
    fn from(item: T) -> Self {
        Self { item, rect: None }
    }
}

type Callback<T, Cursor: Clone + Debug> = Box<dyn FnOnce(Vec<T>, Option<Cursor>)>;
type Loader<T: Debug, Cursor: Clone + Debug> = Box<dyn FnMut(Option<Cursor>, Callback<T, Cursor>)>;

pub struct InfiniteScroll<T: Debug, Cursor: Clone + Debug> {
    id: Id,
    start_loader: Option<Loader<T, Cursor>>,
    end_loader: Option<Loader<T, Cursor>>,

    start_cursor: Option<Cursor>,
    end_cursor: Option<Cursor>,

    top_loading_state: LoadingState<T, Cursor>,
    bottom_loading_state: LoadingState<T, Cursor>,

    items: Vec<T>,

    rows: Vec<RowData>,

    top_inbox: UiInbox<LoadingState<T, Cursor>>,
    bottom_inbox: UiInbox<LoadingState<T, Cursor>>,

    previous_item_range: Range<usize>,

    // The index of the first item that has an unknown rect
    last_known_row_index: Option<usize>,

    average_row_size: Option<Vec2>,
    average_items_per_row: Option<f32>,

    // We will recalculate every item's rect if the scroll area's width changes
    last_width: f32,

    filter: Option<Box<dyn Fn(&T) -> bool + Send + Sync>>,
}

impl<T: Debug, Cursor: Clone + Debug> Debug for InfiniteScroll<T, Cursor> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("InfiniteScroll { ... }")
    }
}

impl<T: Debug + 'static, Cursor: Clone + Debug + 'static> InfiniteScroll<T, Cursor> {
    pub fn new(id: &str) -> Self {
        let top_inbox = UiInbox::new();
        let bottom_inbox = UiInbox::new();
        let id = Id::new(id);
        Self {
            id,
            start_loader: None,
            end_loader: None,
            start_cursor: None,
            end_cursor: None,
            top_loading_state: LoadingState::Idle,
            bottom_loading_state: LoadingState::Idle,
            bottom_inbox,
            top_inbox,
            items: vec![],
            previous_item_range: usize::MAX..usize::MAX,
            last_known_row_index: None,
            last_width: 0.0,
            average_row_size: None,
            rows: vec![],
            average_items_per_row: None,
            filter: None,
        }
    }

    pub fn start_loader<F: FnMut(Option<Cursor>, Callback<T, Cursor>) + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.start_loader = Some(Box::new(f));
        self
    }

    pub fn end_loader<F: FnMut(Option<Cursor>, Callback<T, Cursor>) + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.end_loader = Some(Box::new(f));
        self
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.rows.clear();
        self.last_known_row_index = None;
        self.previous_item_range = usize::MAX..usize::MAX;
        self.average_row_size = None;
        self.average_items_per_row = None;
        self.last_width = 0.0;
        self.top_loading_state = LoadingState::Idle;
        self.bottom_loading_state = LoadingState::Idle;

        // Create new inboxes in case there is a request in progress
        self.top_inbox = UiInbox::new();
        self.bottom_inbox = UiInbox::new();
    }

    pub fn set_filter(&mut self, filter: impl Fn(&T) -> bool + Send + Sync + 'static) {
        self.filter = Some(Box::new(filter));
        self.previous_item_range = usize::MAX..usize::MAX;
        self.rows.clear();
    }

    fn read_inboxes(&mut self, ui: &mut Ui) {
        self.bottom_inbox.read(ui).for_each(|state| {
            self.bottom_loading_state = match state {
                LoadingState::Loaded(items, cursor) => {
                    self.end_cursor = cursor;
                    let empty = items.is_empty();
                    self.items.extend(items);
                    if empty {
                        LoadingState::NoMoreItems
                    } else {
                        LoadingState::Idle
                    }
                }
                state => state,
            };
        });

        self.top_inbox.read(ui).for_each(|state| {
            self.top_loading_state = match state {
                LoadingState::Loaded(mut items, cursor) => {
                    self.start_cursor = cursor;
                    let empty = items.is_empty();
                    let mut old_items = mem::take(&mut self.items);
                    self.items = items;
                    self.items.append(&mut old_items);
                    if empty {
                        LoadingState::NoMoreItems
                    } else {
                        LoadingState::Idle
                    }
                }
                state => state,
            };
        });
    }

    fn filtered_items<'a>(
        items: &'a mut Vec<T>,
        filter: &Option<Box<dyn Fn(&T) -> bool + Send + Sync>>,
    ) -> Vec<&'a mut T> {
        if let Some(filter) = filter {
            items
                .iter_mut()
                .filter(|item| filter(*item))
                .collect::<Vec<_>>()
        } else {
            items.iter_mut().collect::<Vec<_>>()
        }
    }

    /// The layout function is called with the remaining items and should return the count of items used.
    /// It should return the index of the end of the row.
    pub fn ui_custom_layout(
        &mut self,
        end_prefetch: usize,
        ui: &mut Ui,
        mut layout: impl FnMut(&mut Ui, &mut [&mut T]) -> usize,
    ) {
        self.read_inboxes(ui);

        let item_range = ui
            .scope(|ui| {
                let mut items = Self::filtered_items(&mut self.items, &self.filter);

                if ui.available_width() != self.last_width {
                    self.last_known_row_index = None;
                    self.last_width = ui.available_width();
                    self.rows.clear();
                }

                // Start of the scroll area (!=0 after scrolling)
                let min = ui.next_widget_position();

                let mut row_start_index = self.last_known_row_index.unwrap_or(0);

                // This calculates the visual rect inside the scroll area
                // Should be equivalent to to viewport from ScrollArea::show_viewport()
                let visible_rect = ui.clip_rect().translate(-ui.min_rect().min.to_vec2());

                // Find the first row that is visible
                loop {
                    if row_start_index == 0 {
                        break;
                    }

                    if let Some(row) = self.rows.get(row_start_index) {
                        if row.rect.min.y <= visible_rect.min.y {
                            ui.add_space(row.rect.min.y);
                            break;
                        }
                    }
                    row_start_index -= 1;
                }
                let mut current_row = row_start_index;

                let item_start_index = self
                    .rows
                    .get(row_start_index)
                    .map(|row| row.range.start)
                    .unwrap_or(0);

                let mut current_item_index = item_start_index;

                loop {
                    // let item = self.items.get_mut(current_row);
                    if current_item_index < items.len() {
                        let scoped = ui.scope(|ui| layout(ui, &mut items[current_item_index..]));
                        let count = scoped.inner;
                        let rect = scoped.response.rect.translate(-(min.to_vec2()));

                        let range = current_item_index..current_item_index + count;

                        if let Some(row) = self.rows.get_mut(current_row) {
                            row.range = range;
                            row.rect = rect;
                        } else {
                            self.rows.push(RowData {
                                range: current_item_index..current_item_index + count,
                                rect,
                            });
                            self.average_row_size = Some(
                                self.average_row_size
                                    .map(|size| {
                                        (current_row as f32 * size + rect.size())
                                            / (current_row as f32 + 1.0)
                                    })
                                    .unwrap_or(rect.size()),
                            );

                            self.average_items_per_row = Some(
                                self.average_items_per_row
                                    .map(|avg_count| {
                                        (current_row as f32 * avg_count + count as f32)
                                            / (current_row as f32 + 1.0)
                                    })
                                    .unwrap_or(count as f32),
                            );

                            self.last_known_row_index = Some(current_row);
                        }

                        current_item_index += count;

                        if rect.max.y > visible_rect.max.y {
                            break;
                        }
                    } else {
                        break;
                    }

                    current_row += 1;
                }

                let item_range = item_start_index..current_item_index;

                if item_range.end < items.len() {
                    ui.set_min_height(
                        (items.len() - item_range.end) as f32
                            / self.average_items_per_row.unwrap_or(1.0)
                            * self.average_row_size.unwrap_or(Vec2::ZERO).y,
                    );
                } else {
                    ui.add_space(50.0);

                    ui.separator();

                    ui.add_space(50.0);

                    ui.vertical_centered(|ui| match &self.bottom_loading_state {
                        LoadingState::Loading => {
                            ui.spinner();
                        }
                        LoadingState::Idle => {
                            ui.spinner();
                        }
                        LoadingState::NoMoreItems => {
                            ui.label("No more items");
                        }
                        LoadingState::Error(e) => {
                            ui.label(format!("Error: {}", e));
                            if ui.button("Try again").clicked() {
                                self.bottom_loading_state = LoadingState::Idle;
                                ui.ctx().request_repaint();
                            }
                        }
                        _ => {}
                    });

                    ui.add_space(300.0);
                }

                item_range
            })
            .inner;
        self.update_items(item_range, end_prefetch);
    }

    fn update_items(&mut self, item_range: Range<usize>, end_prefetch: usize) {
        let mut items = Self::filtered_items(&mut self.items, &self.filter);

        // for i in self.previous_item_range.start..item_range.start.min(self.previous_item_range.end)
        // {
        //     if let Some(item) = items.get_mut(i) {
        //         item.hidden();
        //     }
        // }
        //
        // for i in item_range.end.max(self.previous_item_range.start)..self.previous_item_range.end {
        //     if let Some(item) = items.get_mut(i) {
        //         item.hidden();
        //     }
        // }
        //
        // for i in self.previous_item_range.end.max(item_range.start)..item_range.end {
        //     if let Some(item) = items.get_mut(i) {
        //         item.visible();
        //     }
        // }
        //
        // for i in item_range.start..self.previous_item_range.start.min(item_range.end) {
        //     if let Some(item) = items.get_mut(i) {
        //         item.visible();
        //     }
        // }

        self.previous_item_range = item_range.clone();

        if item_range.end + end_prefetch >= items.len()
            && matches!(self.bottom_loading_state, LoadingState::Idle)
        {
            self.bottom_loading_state = LoadingState::Loading;
            let inbox = self.bottom_inbox.clone();

            if let Some(end_loader) = &mut self.end_loader {
                end_loader(
                    self.end_cursor.clone(),
                    Box::new(move |items, cursor| {
                        inbox.send(LoadingState::Loaded(items, cursor));
                    }),
                );
            }

            // spawn(async move {
            //     let new_items = loaders.load_bottom(cursor).await;
            //     inbox.send(match new_items {
            //         Ok(items) => LoadingState::Loaded(items),
            //         Err(e) => LoadingState::Error(e.to_string()),
            //     });
            // });
        }
    }

    pub fn ui_columns(
        &mut self,
        columns: usize,
        max_row_height: Option<f32>,
        prefetch_count: usize,
        ui: &mut Ui,
        mut item_ui: impl FnMut(&mut Ui, &mut T),
    ) {
        let max_width = ui.available_width();
        let item_width = max_width / columns as f32
            - (ui.spacing().item_spacing.x / columns as f32 * (columns - 1) as f32);
        self.ui_custom_layout(prefetch_count, ui, |ui, items| {
            let count = items.len().min(columns);
            if let Some(max_row_height) = max_row_height {
                ui.set_max_height(max_row_height);
                ui.set_max_width(max_width);
            }

            ui.horizontal(|ui| {
                for item in items.iter_mut().take(count) {
                    ui.scope(|ui| {
                        ui.set_width(item_width);
                        item_ui(ui, item);
                    });
                }
            });

            count
        });
    }

    pub fn ui(&mut self, ui: &mut Ui, prefetch_count: usize, item_ui: impl FnMut(&mut Ui, &mut T)) {
        self.ui_columns(1, None, prefetch_count, ui, item_ui);
    }

    #[cfg(feature = "egui_extras")]
    pub fn ui_table(
        &mut self,
        table: TableBody,
        prefetch_count: usize,
        row_height: f32,
        mut row_ui: impl FnMut(TableRow, &mut T),
    ) {
        use egui_extras::{TableBody, TableRow};

        self.read_inboxes();

        let mut min_item = 0;
        let mut max_item = 0;

        table.rows(row_height, self.items.len(), |index, row| {
            min_item = min_item.min(index);
            max_item = max_item.max(index);
            let item = &mut self.items[index];
            row_ui(row, item);
        });

        let item_range = min_item..max_item;
        self.update_items(item_range, prefetch_count);
    }
}
