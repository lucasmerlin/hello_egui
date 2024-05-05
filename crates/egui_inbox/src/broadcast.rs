use std::sync::Arc;

use hello_egui_utils::MaybeSend;
use parking_lot::Mutex;

use crate::{UiInbox, UiInboxSender};

/// A broadcast channel that can be used to send messages to multiple receivers.
/// Basically a mpmc version of [crate::UiInbox].
///
/// Internally, this is basically a Vec<UiInboxSender<T>>, so it's not optimized for crazy performance.
/// The goal is to provide a really convenient way to send broadcasts in egui (or other immediate mode GUIs).
///
/// NOTE: This is an unbounded channel, and each receivers queue is only emptied when it is read.
/// So if you don't read from a receiver, it might cause a memory leak. If you send a lot
/// of messages and only show a receiver's ui conditionally, it might make sense to read
/// the receiver in a separate update function. This is demonstrated in the router_login example.
#[derive(Debug, Clone)]
pub struct Broadcast<T> {
    senders: Arc<Mutex<Vec<UiInboxSender<T>>>>,
}

impl<T> Default for Broadcast<T> {
    fn default() -> Self {
        Self {
            senders: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<T> Broadcast<T> {
    /// Create a new broadcast channel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Subscribe to the broadcast channel, receiving a [BroadcastReceiver] of type [T].
    pub fn subscribe(&self) -> BroadcastReceiver<T> {
        let (tx, rx) = UiInbox::channel();
        self.senders.lock().push(tx);
        rx
    }

    /// Send a message of type [T] to all subscribers.
    pub fn send(&self, message: T)
    where
        T: Clone + MaybeSend + 'static,
    {
        let mut senders = self.senders.lock();
        senders.retain(|tx| tx.send(message.clone()).is_ok());
    }
}

/// A receiver for a [Broadcast]. This is currently just a re-export of [crate::UiInbox], but this might
/// change in the future, so make sure you always use this type when you subscribe to a broadcast.
pub type BroadcastReceiver<T> = UiInbox<T>;
