use std::sync::Arc;

use parking_lot::Mutex;
use type_map::concurrent::TypeMap;

use crate::{AsRequestRepaint, RequestRepaintContext, UiInbox, UiInboxSender};

#[derive(Debug)]
struct TypeInboxEntry<T> {
    sender: UiInboxSender<T>,
    inbox: UiInbox<T>,
}

impl<T> TypeInboxEntry<T> {
    fn new() -> Self {
        let (sender, inbox) = UiInbox::channel();
        Self { sender, inbox }
    }
}

#[derive(Debug, Default)]
struct TypeInboxInner {
    map: TypeMap,
    ctx: Option<RequestRepaintContext>,
}

/// A type-map based version of [UiInbox] which can be used to send messages
/// to a component from different parts of the application.
///
/// Use [crate::TypeBroadcast] instead, if you want to send messages to multiple components (mpmc like channel).
#[derive(Clone, Debug, Default)]
pub struct TypeInbox(Arc<Mutex<TypeInboxInner>>);

impl TypeInbox {
    /// Create a new [TypeInbox] with the given [RequestRepaintContext].
    /// Usually, this would be a [egui::Context].
    pub fn new_with_ctx(ctx: impl AsRequestRepaint + 'static) -> Self {
        Self(Arc::new(Mutex::new(TypeInboxInner {
            map: TypeMap::new(),
            ctx: Some(ctx.as_request_repaint()),
        })))
    }

    /// Create a new [TypeInbox].
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(TypeInboxInner {
            map: TypeMap::new(),
            ctx: None,
        })))
    }

    /// Send a message of type [T] to the component.
    /// If the component has a [RequestRepaintContext] attached, a repaint will be requested.
    pub fn send<T: Send + 'static>(&mut self, message: T) {
        let mut guard = self.0.lock();
        let entry = guard.map.entry().or_insert_with(TypeInboxEntry::<T>::new);
        entry.sender.send(message).ok();
        if let Some(ctx) = &guard.ctx {
            ctx.request_repaint();
        }
    }

    /// Read the inbox, returning an iterator over all pending messages.
    pub fn read<T: Send + 'static>(
        &mut self,
        ui: &impl AsRequestRepaint,
    ) -> impl Iterator<Item = T> + '_ {
        let mut guard = self.0.lock();

        if guard.ctx.is_none() {
            guard.ctx = Some(ui.as_request_repaint());
        }

        let iter = guard
            .map
            .entry()
            .or_insert_with(TypeInboxEntry::<T>::new)
            .inbox
            .read_without_ctx();
        iter
    }

    /// Read the inbox without setting the repaint context, returning an iterator over all pending messages.
    /// This can be used with [TypeInbox::new_with_ctx].
    pub fn read_without_ctx<T: Send + 'static>(&mut self) -> impl Iterator<Item = T> + '_ {
        let mut guard = self.0.lock();
        let iter = guard
            .map
            .entry()
            .or_insert_with(TypeInboxEntry::<T>::new)
            .inbox
            .read_without_ctx();
        iter
    }
}
