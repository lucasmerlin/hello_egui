#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::{Debug, Formatter};
use std::mem;
use std::ops::Range;

use egui::Ui;
#[cfg(feature = "egui_extras")]
use egui_extras::{TableBody, TableRow};

use egui_inbox::UiInbox;
use egui_virtual_list::{VirtualList, VirtualListResponse};
use hello_egui_utils::asyncify;

/// The loading state of the infinite scroll, for either the start or end of the list.
#[derive(Debug)]
pub enum LoadingState<T, Cursor> {
    /// The loader has just received more items, which will be added to the list during the current or next frame.
    Loaded(Vec<T>, Option<Cursor>),
    /// The loader is currently loading items.
    Loading,
    /// The loader is currently idle.
    Idle,
    /// The loader has no more items to load.
    NoMoreItems,
    /// The loader has encountered an error.
    Error(String),
}

impl<T, C> LoadingState<T, C> {
    /// Returns true if the state is [`LoadingState::Loading`]
    pub fn loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
}

type CallbackResult<T, Cursor> = Result<(Vec<T>, Option<Cursor>), String>;
type Callback<T, Cursor> = Box<dyn FnOnce(CallbackResult<T, Cursor>) + Send + Sync>;
type Loader<T, Cursor> = Box<dyn FnMut(Option<Cursor>, Callback<T, Cursor>) + Send + Sync>;

type FilterType<T> = Box<dyn Fn(&T) -> bool + Send + Sync>;

/// A infinite scroll widget.
pub struct InfiniteScroll<T: Debug + Send + Sync, Cursor: Clone + Debug> {
    /// Access to the items.
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

    /// The `egui_virtual_list` instance. You can use this to customize settings of the virtual list.
    pub virtual_list: VirtualList,
}

impl<T, Cursor> Debug for InfiniteScroll<T, Cursor>
where
    T: Debug + Send + Sync,
    Cursor: Clone + Debug + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InfiniteScroll")
            .field("items", &self.items)
            .field("start_loader", &self.start_loader.is_some())
            .field("end_loader", &self.end_loader.is_some())
            .field("start_cursor", &self.start_cursor)
            .field("end_cursor", &self.end_cursor)
            .field("top_loading_state", &self.top_loading_state)
            .field("bottom_loading_state", &self.bottom_loading_state)
            .field("top_inbox", &self.top_inbox)
            .field("bottom_inbox", &self.bottom_inbox)
            .field("filter", &self.filter.is_some())
            .field("virtual_list", &self.virtual_list)
            .finish()
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
    /// Create a new infinite scroll.
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

    asyncify! {
        /// Sets the loader for the start of the list.
        start_loader,
        f: (impl FnMut(Callback<T, Cursor>, c: Option<Cursor>,) + Send + Sync + 'static),
        call_prefix: (Self::),
        generics: (),
        async_generics: (<F: std::future::Future<Output = CallbackResult<T, Cursor>> + Send + 'static>),
        parameters: (),
        future: (impl FnMut(Option<Cursor>) -> F + Send + Sync + 'static),
        return_type: (Self),
        body: |(mut self,)| {
            self.start_loader = Some(Box::new(f));
            self
        },
    }

    asyncify! {
        /// Sets the loader for the end of the list.
        end_loader,
        f: (impl FnMut(Callback<T, Cursor>, c: Option<Cursor>,) + Send + Sync + 'static),
        call_prefix: (Self::),
        generics: (),
        async_generics: (<F: std::future::Future<Output = CallbackResult<T, Cursor>> + Send + 'static>),
        parameters: (),
        future: (impl FnMut(Option<Cursor>) -> F + Send + Sync + 'static),
        return_type: (Self),
        body: |(mut self,)| {
            self.end_loader = Some(Box::new(f));
            self
        },
    }

    /// Returns true if the initial loading is in progress (no items and loading state is loading)
    pub fn initial_loading(&self) -> bool {
        self.items.is_empty()
            && (self.top_loading_state.loading() || self.bottom_loading_state.loading())
    }

    /// Returns true if there is a request in progress
    pub fn loading(&self) -> bool {
        self.top_loading_state.loading() || self.bottom_loading_state.loading()
    }

    /// Returns information about the top loading state
    pub fn top_loading_state(&self) -> &LoadingState<T, Cursor> {
        &self.top_loading_state
    }

    /// Returns information about the bottom loading state
    pub fn bottom_loading_state(&self) -> &LoadingState<T, Cursor> {
        &self.bottom_loading_state
    }

    /// Retry loading the top items
    /// This only works if the top loading state is [`LoadingState::Error`]
    pub fn retry_top(&mut self) {
        if let LoadingState::Error(_) = self.top_loading_state {
            self.top_loading_state = LoadingState::Idle;
        }
    }

    /// Retry loading the bottom items
    /// This only works if the bottom loading state is [`LoadingState::Error`]
    pub fn retry_bottom(&mut self) {
        if let LoadingState::Error(_) = self.bottom_loading_state {
            self.bottom_loading_state = LoadingState::Idle;
        }
    }

    /// Resets the infinite scroll, clearing all items and loading states.
    /// This is a alias for [`InfiniteScroll::reload`].
    pub fn reset(&mut self) {
        self.items.clear();
        self.top_loading_state = LoadingState::Idle;
        self.bottom_loading_state = LoadingState::Idle;
        self.start_cursor = None;
        self.end_cursor = None;

        // Create new inboxes in case there is a request in progress
        self.top_inbox = UiInbox::new();
        self.bottom_inbox = UiInbox::new();

        self.virtual_list.reset();
    }

    /// Reset the underlying virtual list.
    /// Call this if a item's height has been modified or you manually inserted items somewhere in the list.
    /// This will only delete the cached heights.
    pub fn reset_virtual_list(&mut self) {
        self.virtual_list.reset();
    }

    /// Resets the infinite scroll, clearing all items and loading states.
    /// This is a alias for [`InfiniteScroll::reset`].
    pub fn reload(&mut self) {
        self.reset();
    }

    /// Use this to filter on the client. Not recommended for large datasets.
    /// If the filter filters enough items, the loader will be called again and again,
    /// until enough items to filter the screen are found or the loader returns no more items.
    /// So in the worst case, this could result in loading *all* items.
    /// The list will update automatically when the filter is set.
    pub fn set_filter(&mut self, filter: impl Fn(&T) -> bool + Send + Sync + 'static) {
        self.filter = Some(Box::new(filter));
        self.virtual_list.reset();
    }

    /// Update the data (check if anything has been loaded) without showing the ui.
    pub fn update(&mut self, ctx: &egui::Context) {
        self.bottom_inbox.read(ctx).for_each(|state| {
            self.bottom_loading_state = match state {
                LoadingState::Loaded(items, cursor) => {
                    let has_cursor = cursor.is_some();
                    if has_cursor {
                        self.end_cursor = cursor;
                    }
                    self.items.extend(items);

                    ctx.request_repaint();
                    if has_cursor {
                        LoadingState::Idle
                    } else {
                        LoadingState::NoMoreItems
                    }
                }
                state => state,
            };
        });

        self.top_inbox.read(ctx).for_each(|state| {
            self.top_loading_state = match state {
                LoadingState::Loaded(items, cursor) => {
                    self.virtual_list.items_inserted_at_start(items.len());
                    let has_cursor = cursor.is_some();
                    if has_cursor {
                        self.start_cursor = cursor;
                    }
                    let mut old_items = mem::take(&mut self.items);
                    self.items = items;
                    self.items.append(&mut old_items);

                    ctx.request_repaint();
                    if has_cursor {
                        LoadingState::Idle
                    } else {
                        LoadingState::NoMoreItems
                    }
                }
                state => state,
            };
        });
    }

    fn filtered_items<'a>(items: &'a mut [T], filter: Option<&FilterType<T>>) -> Vec<&'a mut T> {
        if let Some(filter) = filter {
            items
                .iter_mut()
                .filter(|item| filter(*item))
                .collect::<Vec<_>>()
        } else {
            items.iter_mut().collect::<Vec<_>>()
        }
    }

    /// Custom layout function for the virtual list. You can place items in each row however you please.
    /// The layout function is called with the remaining items and should return the count of items used.
    pub fn ui_custom_layout(
        &mut self,
        ui: &mut Ui,
        end_prefetch: usize,
        mut layout: impl FnMut(&mut Ui, usize, &mut [&mut T]) -> usize,
    ) -> VirtualListResponse {
        self.update(ui.ctx());

        let mut items = Self::filtered_items(&mut self.items, self.filter.as_ref());

        let response = self
            .virtual_list
            .ui_custom_layout(ui, items.len(), |ui, start_index| {
                layout(ui, start_index, &mut items[start_index..])
            });

        self.update_items(&response.item_range, end_prefetch);

        response
    }

    fn update_items(&mut self, item_range: &Range<usize>, end_prefetch: usize) {
        let items = Self::filtered_items(&mut self.items, self.filter.as_ref());

        if item_range.end + end_prefetch >= items.len() {
            self.load_more_end();
        }

        if item_range.start < end_prefetch {
            self.load_more_start();
        }
    }

    /// Load more items at the end (if idle).
    pub fn load_more_end(&mut self) {
        if matches!(self.bottom_loading_state, LoadingState::Idle) {
            if let Some(end_loader) = &mut self.end_loader {
                self.bottom_loading_state = LoadingState::Loading;
                let sender = self.bottom_inbox.sender();
                end_loader(
                    self.end_cursor.clone(),
                    Box::new(move |result| match result {
                        Ok((items, cursor)) => {
                            sender.send(LoadingState::Loaded(items, cursor)).ok();
                        }
                        Err(err) => {
                            sender.send(LoadingState::Error(err.clone())).ok();
                        }
                    }),
                );
            }
        }
    }

    /// Load more items at the start (if idle).
    pub fn load_more_start(&mut self) {
        if matches!(self.top_loading_state, LoadingState::Idle) {
            if let Some(start_loader) = &mut self.start_loader {
                self.top_loading_state = LoadingState::Loading;
                let sender = self.top_inbox.sender();
                start_loader(
                    self.start_cursor.clone(),
                    Box::new(move |result| match result {
                        Ok((items, cursor)) => {
                            sender.send(LoadingState::Loaded(items, cursor)).ok();
                        }
                        Err(err) => {
                            sender.send(LoadingState::Error(err.clone())).ok();
                        }
                    }),
                );
            }
        }
    }

    /// A simple layout with multiple columns.
    /// You can also make it responsive by using eg
    /// `(ui.available_width() / 300.0).ceil() as usize` as the column count.
    pub fn ui_columns(
        &mut self,
        ui: &mut Ui,
        prefetch_count: usize,
        columns: usize,
        max_row_height: Option<f32>,
        mut item_ui: impl FnMut(&mut Ui, usize, &mut T),
    ) {
        let max_width = ui.available_width();
        let item_width = max_width / columns as f32
            - (ui.spacing().item_spacing.x / columns as f32 * (columns - 1) as f32);
        self.ui_custom_layout(ui, prefetch_count, |ui, start_index, items| {
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

    /// A single column layout.
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        prefetch_count: usize,
        mut item_ui: impl FnMut(&mut Ui, usize, &mut T),
    ) {
        self.ui_custom_layout(ui, prefetch_count, |ui, start_index, items| {
            if let Some(item) = items.first_mut() {
                item_ui(ui, start_index, item);
                1
            } else {
                0
            }
        });
    }

    /// Layout for to use with [`egui_extras::TableBody`].
    #[cfg(feature = "egui_extras")]
    pub fn ui_table(
        &mut self,
        mut table: TableBody,
        prefetch_count: usize,
        row_height: f32,
        mut row_ui: impl FnMut(TableRow, &mut T),
    ) {
        self.update(table.ui_mut().ctx());

        let mut min_item = 0;
        let mut max_item = 0;

        table.rows(row_height, self.items.len(), |row| {
            let index = row.index();
            min_item = min_item.min(index);
            max_item = max_item.max(index);
            let item = &mut self.items[index];
            row_ui(row, item);
        });

        let item_range = min_item..max_item;
        self.update_items(&item_range, prefetch_count);
    }
}
