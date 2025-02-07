# egui_inbox changelog

## 0.8.0

- Update egui to 0.31

## 0.7.0

- update egui to 0.30

## 0.6.0

- update egui to 0.29

## 0.5.0

- update egui to 0.28

## 0.4.1

- Add broadcast (mpmc) channel to egui_inbox.
- Add [type-map](https://crates.io/crates/type-map) based versions of `UiInbox` and `Broadcast`.
- Add a complex example [(router_login)](./examples/router_login.rs) showcasing a simple application with different
  independent components interacting with each
  other.

## 0.4.0

- egui_inbox now can be used without egui
    - There is a new trait AsRequestRepaint, which can be implemented for anything that can request a repaint
    - **Breaking**: new_with_ctx now takes a reference to the context
    - **Breaking**: read_without_ui and replace_without_ui have been renamed to read_without_ctx and replace_without_ctx
    - All other methods now take a impl AsRequestRepaint instead of a &Ui
      but this should not break existing code. A benefit is that you can also
      call the methods with a &Context instead of a &Ui now.

- Added `async` and `tokio` features that add the following:
    - `UiInbox::spawn` to conveniently spawn a task that will be cancelled when the inbox is dropped.
    - `UiInbox::spawn_detached` to spawn a task that will not be cancelled when the inbox is dropped.
- updated egui to 0.27

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
