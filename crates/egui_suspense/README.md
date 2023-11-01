# egui_suspense

A helper to display loading, error and retry uis when waiting for asynchronous data.

## Minimal example
```rust no_run
use eframe::egui;
use egui::CentralPanel;
use egui_suspense::EguiSuspense;

pub fn main() -> eframe::Result<()> {
    let mut suspense = EguiSuspense::reloadable(|cb| {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            cb(if rand::random() {
                Ok("Hello".to_string())
            } else {
                Err("OOPSIE WOOPSIE!".to_string())
            });
        });
    });

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                
                // This will show a spinner while loading and an error message with a 
                // retry button if the callback returns an error.
                suspense.ui(ui, |ui, data, state| {
                    ui.label(format!("Data: {:?}", data));

                    if ui.button("Reload").clicked() {
                        state.reload();
                    }
                });
            });
        },
    )
}
```
