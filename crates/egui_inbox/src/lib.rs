use std::fmt::Debug;
use std::mem;
use std::sync::{Arc, Mutex};

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
///                     let mut inbox_clone = inbox.clone();
///                     std::thread::spawn(move || {
///                         std::thread::sleep(std::time::Duration::from_secs(1));
///                         inbox_clone.send(Some("Hello from another thread!".to_string()));
///                     });
///                 }
///             });
///         },
///     )
/// }
/// ```
#[derive(Debug)]
pub struct UiInbox<T: Debug>(Arc<Mutex<State<T>>>);

#[derive(Debug)]
struct State<T: Debug> {
    items: Vec<T>,
    ctx: Option<Context>,
}

impl<T: Debug> Default for UiInbox<T> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(State {
            items: Vec::new(),
            ctx: None,
        })))
    }
}

impl<T: Debug> Clone for UiInbox<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: Debug> UiInbox<T> {
    /// Create a new inbox.
    /// The context is grabbed from the [Ui] passed to [UiInbox::read], so
    /// if you call [UiInbox::send] before [UiInbox::read], no repaint is requested.
    /// If you want to set the context on creation, use [UiInbox::new_with_ctx].
    pub fn new() -> Self {
        Default::default()
    }

    /// Create a new inbox with a context.
    pub fn new_with_ctx(ctx: Context) -> Self {
        Self(Arc::new(Mutex::new(State {
            items: Vec::new(),
            ctx: Some(ctx),
        })))
    }

    /// Set the [Context] to use for requesting repaints.
    /// Usually this is not needed, since the [Context] is grabbed from the [Ui] passed to [UiInbox::read].
    pub fn set_ctx(&mut self, ctx: Context) {
        let mut guard = self.0.lock().unwrap();
        guard.ctx = Some(ctx);
    }

    /// Send an item to the inbox.
    /// Calling this will request a repaint from egui.
    /// If this is called before a call to `UiInbox::read` was done, no repaint is requested
    /// (Since we didn't have a chance to get a reference to [Context] yet).
    pub fn send(&self, item: T) {
        let mut guard = self.0.lock().unwrap();
        guard.items.push(item);
        if let Some(ctx) = &guard.ctx {
            ctx.request_repaint();
        }
    }

    /// Returns an iterator over all items sent to the inbox.
    /// The inbox is cleared after this call.
    ///
    /// The ui is only passed here so we can grab a reference to [Context].
    /// This is mostly done for convenience, so you don't have to pass a reference to [Context]
    /// to every struct that uses an inbox on creation.
    pub fn read(&self, ui: &mut Ui) -> impl Iterator<Item = T> {
        let mut inbox = self.0.lock().unwrap();
        if inbox.ctx.is_none() {
            inbox.ctx = Some(ui.ctx().clone());
        }
        mem::take(&mut inbox.items).into_iter()
    }

    /// Same as [UiInbox::read], but you don't need to pass a reference to [Ui].
    /// If you use this, make sure you set the [Context] with [UiInbox::set_ctx] or
    /// [UiInbox::new_with_ctx] manually.
    pub fn read_without_ui(&self) -> impl Iterator<Item = T> {
        let mut inbox = self.0.lock().unwrap();
        mem::take(&mut inbox.items).into_iter()
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
        let mut inbox = self.0.lock().unwrap();
        if inbox.ctx.is_none() {
            inbox.ctx = Some(ui.ctx().clone());
        }
        let data = mem::take(&mut inbox.items);
        if let Some(item) = data.into_iter().last() {
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
        let mut inbox = self.0.lock().unwrap();
        let data = mem::take(&mut inbox.items);
        if let Some(item) = data.into_iter().last() {
            *target = item;
            true
        } else {
            false
        }
    }
}
