# egui_inbox changelog

## 0.3.0
- update egui to 0.26

## 0.2.0
This update changes `UiInbox` to act more like a channel. I've removed the possibility to 
send messages via the `UiInbox` itself, and instead added a `UiInboxSender` that can be used to send messages.
Use `UiInbox::sender` to get a `UiInboxSender` or create the inbox with `UiInbox::channel` to get a tuple of
sender and receiver, like it works with other channels.

The main reason for this change is, that the senders can now be notified, once the inbox has been dropped,
so that they can stop whatever background work they are doing.

Other changes:
- Updated egui to 0.25, dropping support for older versions