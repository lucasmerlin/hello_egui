# egui_inbox

[![egui_ver](https://img.shields.io/badge/egui-0.26.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_inbox.svg)](https://crates.io/crates/egui_inbox)
[![Documentation](https://docs.rs/egui_inbox/badge.svg)](https://docs.rs/egui_inbox)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_inbox.svg)](https://crates.io/crates/egui_inbox)



[content]:<>


Channel to send messages to egui views from async functions, callbacks, etc. without having to use interior mutability.
Will automatically call `request_repaint()` on the `Ui` when a message is received.

Example:
    
```rust no_run
use eframe::egui;
use egui::CentralPanel;
use egui_inbox::UiInbox;

pub fn main() -> eframe::Result<()> {
    let inbox = UiInbox::new();
    let mut state = None;

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                // `read` will return an iterator over all pending messages
                if let Some(last) = inbox.read(ui).last() {
                    state = last;
                }
                // There also is a `replace` method that you can use as a shorthand for the above:
                // inbox.replace(ui, &mut state);

                ui.label(format!("State: {:?}", state));
                if ui.button("Async Task").clicked() {
                    state = Some("Waiting for async task to complete".to_string());
                    let tx = inbox.sender();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        // Send will return an error if the receiver has been dropped
                        // but unless you have a long running task that will send multiple messages
                        // you can just ignore the error
                        tx.send(Some("Hello from another thread!".to_string())).ok();
                    });
                }
            });
        },
    )
}
```
