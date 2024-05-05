use std::sync::Arc;

use hello_egui_utils::MaybeSend;
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

#[derive(Debug)]
struct TypeInboxInner {
    map: TypeMap,
    ctx: RequestRepaintContext,
}

/// A type-map based version of [UiInbox] which can be used to send messages
/// to a component from different parts of the application.
///
/// Use [crate::TypeBroadcast] instead, if you want to send messages to multiple components (mpmc like channel).
#[derive(Clone, Debug)]
pub struct TypeInbox(Arc<Mutex<TypeInboxInner>>);

impl TypeInbox {
    /// Create a new [TypeInbox] with the given [RequestRepaintContext].
    /// Usually, this would be a [egui::Context].
    pub fn new(ctx: impl AsRequestRepaint + 'static) -> Self {
        Self(Arc::new(Mutex::new(TypeInboxInner {
            map: TypeMap::new(),
            ctx: ctx.as_request_repaint(),
        })))
    }

    /// Send a message of type [T].
    /// A repaint will be requested.
    pub fn send<T: MaybeSend + 'static>(&self, message: T) {
        let mut guard = self.0.lock();
        let entry = guard.map.entry().or_insert_with(TypeInboxEntry::<T>::new);
        entry.sender.send(message).ok();
        guard.ctx.request_repaint();
    }

    /// Read the inbox, returning an iterator over all pending messages.
    pub fn read<T: MaybeSend + 'static>(&self) -> impl Iterator<Item = T> {
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
