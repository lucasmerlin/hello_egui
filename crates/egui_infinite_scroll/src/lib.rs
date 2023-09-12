use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::Range;

use egui::Ui;

#[cfg(feature = "egui_extras")]
use egui_extras::{TableBody, TableRow};
use egui_inbox::UiInbox;
use egui_virtual_list::{VirtualList, VirtualListResponse};

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

type Callback<T, Cursor> = Box<dyn FnOnce(Result<(Vec<T>, Option<Cursor>), String>) + Send + Sync>;
type Loader<T, Cursor> = Box<dyn FnMut(Option<Cursor>, Callback<T, Cursor>) + Send + Sync>;

type FilterType<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

pub struct InfiniteScroll<T: Debug + Send + Sync, Cursor: Clone + Debug> {
    pub items: Vec<T>,

    start_loader: Option<Loader<T, Cursor>>,
    end_loader: Option<Loader<T, Cursor>>,

    start_cursor: Option<Cursor>,
    end_cursor: Option<Cursor>,

    top_loading_state: LoadingState<T, Cursor>,
    bottom_loading_state: LoadingState<T, Cursor>,

    top_inbox: UiInbox<LoadingState<T, Cursor>>,
    bottom_inbox: UiInbox<LoadingState<T, Cursor>>,

    filter: Option<FilterType<T>>,

    virtual_list: VirtualList,
}

impl<T: Debug + Send + Sync, Cursor: Clone + Debug> Debug for InfiniteScroll<T, Cursor> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("InfiniteScroll { ... }")
    }
}

impl<T, Cursor> Default for InfiniteScroll<T, Cursor>
where
    T: Debug + Send + Sync + 'static,
    Cursor: Clone + Debug + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug + Send + Sync + 'static, Cursor: Clone + Debug + Send + 'static>
    InfiniteScroll<T, Cursor>
{
    pub fn new() -> Self {
        let top_inbox = UiInbox::new();
        let bottom_inbox = UiInbox::new();
        Self {
            items: Vec::new(),
            start_loader: None,
            end_loader: None,
            start_cursor: None,
            end_cursor: None,
            top_loading_state: LoadingState::Idle,
            bottom_loading_state: LoadingState::Idle,
            bottom_inbox,
            top_inbox,
            filter: None,
            virtual_list: VirtualList::new(),
        }
    }

    pub fn start_loader<F: FnMut(Option<Cursor>, Callback<T, Cursor>) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.start_loader = Some(Box::new(f));
        self
    }

    pub fn end_loader<F: FnMut(Option<Cursor>, Callback<T, Cursor>) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.end_loader = Some(Box::new(f));
        self
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.top_loading_state = LoadingState::Idle;
        self.bottom_loading_state = LoadingState::Idle;

        // Create new inboxes in case there is a request in progress
        self.top_inbox = UiInbox::new();
        self.bottom_inbox = UiInbox::new();

        self.virtual_list.reset();
    }

    pub fn set_filter(&mut self, filter: impl Fn(&T) -> bool + Send + Sync + 'static) {
        self.filter = Some(Box::new(filter));
        self.virtual_list.reset();
    }

    fn read_inboxes(&mut self, ui: &mut Ui) {
        self.bottom_inbox.read(ui).for_each(|state| {
            self.bottom_loading_state = match state {
                LoadingState::Loaded(items, cursor) => {
                    let has_cursor = cursor.is_some();
                    if has_cursor {
                        self.end_cursor = cursor;
                    }
                    let empty = items.is_empty();
                    self.items.extend(items);
                    if empty || !has_cursor {
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
                LoadingState::Loaded(items, cursor) => {
                    let has_cursor = cursor.is_some();
                    if has_cursor {
                        self.start_cursor = cursor;
                    }
                    let empty = items.is_empty();
                    let mut old_items = mem::take(&mut self.items);
                    self.items = items;
                    self.items.append(&mut old_items);
                    if empty || !has_cursor {
                        LoadingState::NoMoreItems
                    } else {
                        LoadingState::Idle
                    }
                }
                state => state,
            };
        });
    }

    fn filtered_items<'a>(items: &'a mut [T], filter: &Option<FilterType<T>>) -> Vec<&'a mut T> {
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
        mut layout: impl FnMut(&mut Ui, usize, &mut [&mut T]) -> usize,
    ) -> VirtualListResponse {
        self.read_inboxes(ui);

        let mut items = Self::filtered_items(&mut self.items, &self.filter);

        let response = self
            .virtual_list
            .ui_custom_layout(ui, items.len(), |ui, start_index| {
                layout(ui, start_index, &mut items[start_index..])
            });

        ui.add_space(20.0);

        ui.separator();

        ui.add_space(20.0);

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

        ui.add_space(20.0);

        self.update_items(&response.item_range, end_prefetch);

        response
    }

    fn update_items(&mut self, item_range: &Range<usize>, end_prefetch: usize) {
        let items = Self::filtered_items(&mut self.items, &self.filter);

        if item_range.end + end_prefetch >= items.len()
            && matches!(self.bottom_loading_state, LoadingState::Idle)
        {
            self.bottom_loading_state = LoadingState::Loading;
            let inbox = self.bottom_inbox.clone();

            if let Some(end_loader) = &mut self.end_loader {
                end_loader(
                    self.end_cursor.clone(),
                    Box::new(move |result| match result {
                        Ok((items, cursor)) => {
                            inbox.send(LoadingState::Loaded(items, cursor));
                        }
                        Err(err) => {
                            inbox.send(LoadingState::Error(err.to_string()));
                        }
                    }),
                );
            }
        }
    }

    pub fn ui_columns(
        &mut self,
        columns: usize,
        max_row_height: Option<f32>,
        prefetch_count: usize,
        ui: &mut Ui,
        mut item_ui: impl FnMut(&mut Ui, usize, &mut T),
    ) {
        let max_width = ui.available_width();
        let item_width = max_width / columns as f32
            - (ui.spacing().item_spacing.x / columns as f32 * (columns - 1) as f32);
        self.ui_custom_layout(prefetch_count, ui, |ui, start_index, items| {
            let count = items.len().min(columns);
            if let Some(max_row_height) = max_row_height {
                ui.set_max_height(max_row_height);
                ui.set_max_width(max_width);
            }

            ui.horizontal(|ui| {
                for (index, item) in items.iter_mut().enumerate().take(count) {
                    ui.scope(|ui| {
                        ui.set_width(item_width);
                        item_ui(ui, start_index + index, item);
                    });
                }
            });

            count
        });
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        prefetch_count: usize,
        mut item_ui: impl FnMut(&mut Ui, usize, &mut T),
    ) {
        self.ui_custom_layout(prefetch_count, ui, |ui, start_index, items| {
            if let Some(item) = items.first_mut() {
                item_ui(ui, start_index, item);
                1
            } else {
                0
            }
        });
    }

    #[cfg(feature = "egui_extras")]
    pub fn ui_table(
        &mut self,
        mut table: TableBody,
        prefetch_count: usize,
        row_height: f32,
        mut row_ui: impl FnMut(TableRow, &mut T),
    ) {
        self.read_inboxes(table.ui_mut());

        let mut min_item = 0;
        let mut max_item = 0;

        table.rows(row_height, self.items.len(), |index, row| {
            min_item = min_item.min(index);
            max_item = max_item.max(index);
            let item = &mut self.items[index];
            row_ui(row, item);
        });

        let item_range = min_item..max_item;
        self.update_items(&item_range, prefetch_count);
    }
}
