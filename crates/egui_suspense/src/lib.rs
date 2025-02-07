#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::{Debug, Display};

use egui::Ui;

use egui_inbox::UiInbox;
use hello_egui_utils::{asyncify, CallbackType, MaybeSend, MaybeSync};
#[cfg(target_arch = "wasm32")]
mod types {
    use crate::State;
    use egui::Ui;

    pub type CallbackFn<T> = dyn FnOnce(T);
    pub type ReloadFn<T, E> = dyn FnMut(Box<CallbackFn<Result<T, E>>>);
    pub type ErrorUiFn<E> = dyn Fn(&mut Ui, &E, &mut State<'_>);
    pub type LoadingUiFn = dyn Fn(&mut Ui);
    pub type ReloadFnRef<'a> = &'a mut (dyn FnMut());
}
#[cfg(not(target_arch = "wasm32"))]
mod types {
    use crate::State;
    use egui::Ui;

    pub type CallbackFn<T> = dyn FnOnce(T) + Send + Sync;
    pub type ReloadFn<T, E> = dyn FnMut(Box<CallbackFn<Result<T, E>>>) + Send + Sync;
    pub type ErrorUiFn<E> = dyn Fn(&mut Ui, &E, &mut State<'_>) + Send + Sync;
    pub type LoadingUiFn = dyn Fn(&mut Ui) + Send + Sync;
    pub type ReloadFnRef<'a> = &'a mut (dyn FnMut() + Send + Sync);
}

use types::{ErrorUiFn, LoadingUiFn, ReloadFn, ReloadFnRef};

/// Helper struct to call the reload function.
pub struct State<'a> {
    /// True if this is a reloadable suspense.
    pub reloadable: bool,
    reload_fn: ReloadFnRef<'a>,
}

impl State<'_> {
    /// Call this to reload the data.
    pub fn reload(&mut self) {
        (self.reload_fn)();
    }
}

/// A widget that shows a spinner while data is loading and shows
/// an error message and retry button if the data failed to load.
pub struct EguiSuspense<T, E: Display + Debug = String> {
    inbox: UiInbox<Result<T, E>>,
    data: Option<Result<T, E>>,

    reload_fn: Option<Box<ReloadFn<T, E>>>,

    error_ui: Option<Box<ErrorUiFn<E>>>,
    loading_ui: Option<Box<LoadingUiFn>>,
}

impl<T: Debug, E: Display + Debug> Debug for EguiSuspense<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiSuspense").finish()
    }
}

impl<T: MaybeSend + MaybeSync + 'static, E: Display + Debug + MaybeSend + MaybeSync + 'static>
    EguiSuspense<T, E>
{
    asyncify!(
        /// Create a new suspense that can be reloaded.
        reloadable,
        callback_mut: (impl FnMut(CallbackType<Result<T, E>>, ) + MaybeSend + MaybeSync + 'static),
        call_prefix: (Self::),
        generics: (),
        async_generics: (<F: std::future::Future<Output = Result<T, E>> + MaybeSend + 'static>),
        parameters: (),
        future: impl FnMut() -> F + MaybeSend + MaybeSync + 'static,
        return_type: (Self),
        body: |()| {
            let mut callback_mut = callback_mut;
            let inbox = UiInbox::new();
            let inbox_clone = inbox.sender();
            callback_mut(Box::new(move |result| {
                inbox_clone.send(result).ok();
            }));
            Self {
                inbox,
                data: None,

                reload_fn: Some(Box::new(callback_mut)),
                error_ui: None,
                loading_ui: Some(Box::new(|ui| {
                    ui.spinner();
                })),
            }
        },
    );

    asyncify!(
        /// Create a new suspense that will only try to load the data once.
        single_try,
        callback_once: (impl FnOnce(CallbackType<Result<T, E>>) + MaybeSend + MaybeSync + 'static),
        call_prefix: (Self::),
        generics: (),
        async_generics: (<F: std::future::Future<Output = Result<T, E>> + MaybeSend + MaybeSync + 'static>),
        parameters: (),
        future: F,
        return_type: (Self),
        body: |()| {
            let inbox = UiInbox::new();
            let inbox_clone = inbox.sender();
            callback_once(Box::new(move |result| {
                inbox_clone.send(result).ok();
            }));
            Self {
                inbox,
                data: None,

                reload_fn: None,
                error_ui: None,
                loading_ui: Some(Box::new(|ui| {
                    ui.spinner();
                })),
            }
        },
    );

    /// Create a new suspense that is already loaded.
    pub fn loaded(data: T) -> Self {
        Self {
            inbox: UiInbox::new(),
            data: Some(Ok(data)),

            reload_fn: None,
            error_ui: None,
            loading_ui: None,
        }
    }

    /// Use this to customize the loading ui.
    pub fn loading_ui(mut self, f: impl Fn(&mut Ui) + 'static + MaybeSend + MaybeSync) -> Self {
        self.loading_ui = Some(Box::new(f));
        self
    }

    /// Use this to disable the loading ui.
    /// Nothing will be shown while the data is loading.
    /// Useful when you want to show a loading indicator somewhere else, e.g. when using
    /// [egui_pull_to_refresh](https://crates.io/crates/egui_pull_to_refresh).
    pub fn no_loading_ui(mut self) -> Self {
        self.loading_ui = None;
        self
    }

    /// Use this to customize the error ui.
    /// The closure will be called with the error and a [State] struct.
    pub fn error_ui(
        mut self,
        f: impl Fn(&mut Ui, &E, &mut State<'_>) + 'static + MaybeSend + MaybeSync,
    ) -> Self {
        self.error_ui = Some(Box::new(f));
        self
    }

    /// Show the actual ui.
    /// The content closure will be called with the data and a [State] struct.
    pub fn ui<R>(
        &mut self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui, &mut T, &mut State) -> R,
    ) -> Option<R> {
        let mut result = None;

        if let Some(result) = self.inbox.read(ui).last() {
            self.data = Some(result);
        }

        let mut clear_data = false;
        let clear_data_ref = &mut clear_data;

        match &mut self.data {
            None => {
                if let Some(loading_ui) = &mut self.loading_ui {
                    loading_ui(ui);
                }
            }
            Some(Ok(data)) => {
                let tx = self.inbox.sender();
                result = Some(content(
                    ui,
                    data,
                    &mut State {
                        reloadable: self.reload_fn.is_some(),
                        reload_fn: &mut || {
                            if let Some(reload_fn) = &mut self.reload_fn {
                                *clear_data_ref = true;
                                let tx = tx.clone();
                                reload_fn(Box::new(move |result| {
                                    tx.send(result).ok();
                                }));
                            }
                        },
                    },
                ));
            }
            Some(Err(err)) => {
                if let Some(err_ui) = &mut self.error_ui {
                    let tx = self.inbox.sender();

                    if let Some(reload) = &mut self.reload_fn {
                        err_ui(
                            ui,
                            err,
                            &mut State {
                                reloadable: true,
                                reload_fn: &mut move || {
                                    *clear_data_ref = true;

                                    let inbox = tx.clone();
                                    reload(Box::new(move |result| {
                                        inbox.send(result).ok();
                                    }));
                                },
                            },
                        );
                    } else {
                        err_ui(
                            ui,
                            err,
                            &mut State {
                                reloadable: false,
                                reload_fn: &mut || {},
                            },
                        );
                    }
                } else {
                    ui.label("Something went wrong:");
                    ui.group(|ui| {
                        ui.label(err.to_string());
                    });
                    if let Some(retry_fn) = &mut self.reload_fn {
                        if ui.button("Retry").clicked() {
                            self.data = None;
                            let tx = self.inbox.sender();
                            retry_fn(Box::new(move |result| {
                                tx.send(result).ok();
                            }));
                        }
                    }
                }
            }
        }

        if clear_data {
            self.data = None;
        }

        result
    }

    /// Reload the data.
    /// If this is a [`Self::single_try`], this does nothing.
    pub fn reload(&mut self) {
        if let Some(reload_fn) = &mut self.reload_fn {
            self.data = None;
            let tx = self.inbox.sender();
            reload_fn(Box::new(move |result| {
                tx.send(result).ok();
            }));
        }
    }

    /// Returns true if the data is loading.
    pub fn loading(&self) -> bool {
        self.data.is_none()
    }

    /// Returns true if the data failed to load.
    pub fn has_error(&self) -> bool {
        self.data.as_ref().is_some_and(std::result::Result::is_err)
    }

    /// Returns the data if it is loaded.
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref().and_then(|r| r.as_ref().ok())
    }

    /// Returns the data if it is loaded.
    pub fn data_mut(&mut self) -> Option<&mut T> {
        self.data.as_mut().and_then(|r| r.as_mut().ok())
    }

    /// Returns the error if the data failed to load.
    pub fn error(&self) -> Option<&E> {
        self.data.as_ref().and_then(|r| r.as_ref().err())
    }
}
