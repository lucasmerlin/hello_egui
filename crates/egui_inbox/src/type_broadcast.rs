use std::sync::Arc;

use hello_egui_utils::MaybeSend;
use parking_lot::Mutex;
use type_map::concurrent::TypeMap;

use crate::broadcast::{Broadcast, BroadcastReceiver};

/// A broadcast based on [type_map], which can be used to handle broadcasts between different parts of the application.
/// Call [TypeBroadcast::subscribe] to subscribe to a broadcast, receiving a [BroadcastReceiver].
/// Call [TypeBroadcast::send] to send a message to all subscribers.
///
/// Use [crate::type_inbox::TypeInbox] instead, if you want to send messages to specific components (mpsc like channel).
#[derive(Debug, Clone, Default)]
pub struct TypeBroadcast {
    broadcasts: Arc<Mutex<TypeMap>>,
}

impl TypeBroadcast {
    /// Create a new [TypeBroadcast].
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to a broadcast, receiving a [BroadcastReceiver] of type [T].
    pub fn subscribe<T: MaybeSend + 'static>(&self) -> BroadcastReceiver<T> {
        self.broadcasts
            .lock()
            .entry()
            .or_insert_with(|| Broadcast::new())
            .subscribe()
    }

    /// Send a message of type [T] to all subscribers.
    /// If there are any subscribers with a [crate::RequestRepaintContext] attached, a repaint will be requested.
    pub fn send<T: MaybeSend + Clone + 'static>(&self, message: T) {
        let mut broadcasts = self.broadcasts.lock();
        let entry = broadcasts.entry().or_insert_with(|| Broadcast::new());
        entry.send(message);
    }
}
