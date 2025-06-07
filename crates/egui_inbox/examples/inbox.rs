use eframe::{egui, NativeOptions};
use egui::CentralPanel;
use egui_inbox::UiInbox;

pub fn main() -> eframe::Result<()> {
    let inbox = UiInbox::new();
    let mut state = None;

    eframe::run_simple_native(
        "DnD Simple Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                // `read` will return an iterator over all pending messages
                if let Some(last) = inbox.read(ui).last() {
                    state = last;
                }
                // There also is a `replace` method that you can use as a shorthand for the above:
                // inbox.replace(ui, &mut state);

                ui.label(format!("State: {state:?}"));
                if ui.button("Async Task").clicked() {
                    state = Some("Waiting for async task to complete".to_owned());
                    let tx = inbox.sender();
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        // Send will return an error if the receiver has been dropped
                        // but unless you have a long running task that will send multiple messages
                        // you can just ignore the error
                        tx.send(Some("Hello from another thread!".to_owned())).ok();
                    });
                }
            });
        },
    )
}
