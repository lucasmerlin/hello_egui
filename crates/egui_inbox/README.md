# egui_inbox

Utility to send messages to egui views from async functions, callbacks, etc. without having to use interior mutability.

Example:
    
```rust
use eframe::egui;
use egui::CentralPanel;
use egui_inbox::UiInbox;

pub fn main() -> eframe::Result<()> {
    let mut inbox = UiInbox::new();
    let mut state = None;

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                inbox.replace(ui, &mut state);

                ui.label(format!("State: {:?}", state));
                if ui.button("Async Task").clicked() {
                    state = Some("Waiting for async task to complete".to_string());
                    let mut inbox_clone = inbox.clone();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        inbox_clone.send(Some("Hello from another thread!".to_string()));
                    });
                }
            });
        },
    )
}
```
