#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::Debug;
use std::mem;
use std::sync::Arc;

use egui::mutex::Mutex;
use egui::{Context, Ui};

/// Utility to send messages to egui views from async functions, callbacks, etc. without
/// having to use interior mutability.
/// Example:
/// ```no_run
/// use eframe::egui;
/// use egui::CentralPanel;
/// use egui_inbox::UiInbox;
///
/// pub fn main() -> eframe::Result<()> {
///     let mut inbox = UiInbox::new();
///     let mut state = None;
///
///     eframe::run_simple_native(
///         "DnD Simple Example",
///         Default::default(),
///         move |ctx, _frame| {
///             CentralPanel::default().show(ctx, |ui| {
///                 inbox.replace(ui, &mut state);
///
///                 ui.label(format!("State: {:?}", state));
///                 if ui.button("Async Task").clicked() {
///                     state = Some("Waiting for async task to complete".to_string());
///                     let mut sender = inbox.sender();
///                     std::thread::spawn(move || {
///                         std::thread::sleep(std::time::Duration::from_secs(1));
///                         sender.send(Some("Hello from another thread!".to_string())).ok();
///                     });
///                 }
///             });
///         },
///     )
/// }
/// ```

pub struct UiInbox<T> {
    state: Arc<Mutex<State<T>>>,
    #[cfg(feature = "async")]
    oneshot_channels: Vec<futures_channel::oneshot::Sender<()>>,
}

impl<T> Debug for UiInbox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiInbox").finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct State<T> {
    ctx: Option<Context>,
    queue: Vec<T>,
    dropped: bool,
}

impl<T> State<T> {
    fn new(ctx: Option<Context>) -> Self {
        Self {
            ctx,
            queue: Vec::new(),
            dropped: false,
        }
    }
}

/// Sender for [UiInbox].
pub struct UiInboxSender<T> {
    state: Arc<Mutex<State<T>>>,
}

impl<T> Debug for UiInboxSender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiInboxSender").finish_non_exhaustive()
    }
}

impl<T> Clone for UiInboxSender<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

impl<T> Default for UiInbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for UiInbox<T> {
    fn drop(&mut self) {
        #[cfg(feature = "async")]
        {
            self.oneshot_channels.drain(..).for_each(|tx| {
                tx.send(()).ok();
            });
        }

        let mut state = self.state.lock();
        state.dropped = true;
    }
}

impl<T> UiInbox<T> {
    /// Create a new inbox.
    /// The context is grabbed from the [Ui] passed to [UiInbox::read], so
    /// if you call [UiInbox::send] before [UiInbox::read], no repaint is requested.
    /// If you want to set the context on creation, use [UiInbox::new_with_ctx].
    pub fn new() -> Self {
        Self::_new(None)
    }

    /// Create a new inbox with a context.
    pub fn new_with_ctx(ctx: Context) -> Self {
        Self::_new(Some(ctx))
    }

    fn _new(ctx: Option<Context>) -> Self {
        let state = Arc::new(Mutex::new(State::new(ctx)));
        Self {
            state,
            #[cfg(feature = "async")]
            oneshot_channels: Vec::new(),
        }
    }

    /// Create a inbox and a sender for it.
    pub fn channel() -> (UiInboxSender<T>, Self) {
        let inbox = Self::new();
        let sender = inbox.sender();
        (sender, inbox)
    }

    /// Create a inbox with a context and a sender for it.
    pub fn channel_with_ctx(ctx: Context) -> (UiInboxSender<T>, Self) {
        let inbox = Self::new_with_ctx(ctx);
        let sender = inbox.sender();
        (sender, inbox)
    }

    /// Set the [Context] to use for requesting repaints.
    /// Usually this is not needed, since the [Context] is grabbed from the [Ui] passed to [UiInbox::read].
    pub fn set_ctx(&mut self, ctx: Context) {
        self.state.lock().ctx = Some(ctx);
    }

    /// Returns an iterator over all items sent to the inbox.
    /// The inbox is cleared after this call.
    ///
    /// The ui is only passed here so we can grab a reference to [Context].
    /// This is mostly done for convenience, so you don't have to pass a reference to [Context]
    /// to every struct that uses an inbox on creation.
    pub fn read(&self, ui: &mut Ui) -> impl Iterator<Item = T> {
        let mut state = self.state.lock();
        if state.ctx.is_none() {
            state.ctx = Some(ui.ctx().clone());
        }
        mem::take(&mut state.queue).into_iter()
    }

    /// Same as [UiInbox::read], but you don't need to pass a reference to [Ui].
    /// If you use this, make sure you set the [Context] with [UiInbox::set_ctx] or
    /// [UiInbox::new_with_ctx] manually.
    pub fn read_without_ui(&self) -> impl Iterator<Item = T> {
        let mut state = self.state.lock();
        mem::take(&mut state.queue).into_iter()
    }

    /// Replaces the value of `target` with the last item sent to the inbox.
    /// Any other updates are discarded.
    /// If no item was sent to the inbox, `target` is not updated.
    /// Returns `true` if `target` was updated.
    ///
    /// The ui is only passed here so we can grab a reference to [Context].
    /// This is mostly done for convenience, so you don't have to pass a reference to [Context]
    /// to every struct that uses an inbox on creation.
    pub fn replace(&self, ui: &mut Ui, target: &mut T) -> bool {
        let mut state = self.state.lock();
        if state.ctx.is_none() {
            state.ctx = Some(ui.ctx().clone());
        }

        let item = mem::take(&mut state.queue).pop();
        if let Some(item) = item {
            *target = item;
            true
        } else {
            false
        }
    }

    /// Same as [UiInbox::replace], but you don't need to pass a reference to [Ui].
    /// If you use this, make sure you set the [Context] with [UiInbox::set_ctx] or
    /// [UiInbox::new_with_ctx] manually.
    pub fn replace_without_ui(&self, target: &mut T) -> bool {
        let mut state = self.state.lock();
        let item = mem::take(&mut state.queue).pop();
        if let Some(item) = item {
            *target = item;
            true
        } else {
            false
        }
    }

    /// Returns a sender for this inbox.
    pub fn sender(&self) -> UiInboxSender<T> {
        UiInboxSender {
            state: self.state.clone(),
        }
    }
}

#[cfg(feature = "async")]
mod async_impl {
    use std::pin::pin;

    use futures::{select, FutureExt};

    use hello_egui_utils::spawn;

    use crate::{UiInbox, UiInboxSender};

    impl<T> UiInbox<T> {
        /// Spawns a future that will automatically be cancelled when the inbox is dropped.
        /// Make sure your future is safe to cancel (It may stop at any await point).
        ///
        /// If you want to spawn a future that should definitely run to completion, use [UiInbox::spawn_detached] instead.
        pub fn spawn<F>(&mut self, f: impl FnOnce(UiInboxSender<T>) -> F)
        where
            F: std::future::Future<Output = ()> + Send + 'static,
        {
            let (tx, mut rx) = futures_channel::oneshot::channel();
            self.oneshot_channels.push(tx);

            let sender = self.sender();
            let future = f(sender).fuse();

            spawn(async move {
                let mut future = pin!(future);

                select! {
                    _ = future => {},
                    _ = rx => {},
                }
            });
        }

        /// Spawns a future that will **not** be cancelled when the inbox is dropped.
        pub fn spawn_detached<F>(&mut self, f: impl FnOnce(UiInboxSender<T>) -> F)
        where
            F: std::future::Future<Output = ()> + Send + 'static,
        {
            let sender = self.sender();
            let future = f(sender);
            spawn(future);
        }
    }
}

impl<T> UiInboxSender<T> {
    /// Send an item to the inbox.
    /// Calling this will request a repaint from egui.
    /// If this is called before a call to `UiInbox::read` was done, no repaint is requested
    /// (Since we didn't have a chance to get a reference to [Context] yet).
    ///
    /// This returns an error if the inbox was dropped.
    pub fn send(&self, item: T) -> Result<(), SendError<T>> {
        let mut state = self.state.lock();
        if state.dropped {
            Err(SendError(item))
        } else {
            state.queue.push(item);
            if let Some(ctx) = &state.ctx {
                ctx.request_repaint();
            }
            Ok(())
        }
    }
}

/// Error returned when sending a message to the inbox fails.
/// This can happen if the inbox was dropped.
/// The message is returned in the error, so it can be recovered.
pub struct SendError<T>(pub T);

impl<T: Debug> Debug for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendError").field("item", &self.0).finish()
    }
}
