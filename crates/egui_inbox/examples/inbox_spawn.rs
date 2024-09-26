use eframe::NativeOptions;
use egui::{CentralPanel, Window};
use tokio::time::sleep;

use egui_inbox::UiInbox;

struct MyWindow {
    id: usize,
    count: usize,
    inbox: UiInbox<usize>,
}

struct DropGuard(usize);
impl Drop for DropGuard {
    fn drop(&mut self) {
        println!("Future for window {} was cancelled", self.0);
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let mut id_idx = 0;
    let mut windows: Vec<MyWindow> = Vec::new();

    eframe::run_simple_native(
        "Inbox Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                if ui.button("New Window").clicked() {
                    id_idx += 1;
                    let mut inbox = UiInbox::new();
                    inbox.spawn(|tx| async move {
                        let _guard = DropGuard(id_idx);
                        let mut count = 0;
                        loop {
                            sleep(std::time::Duration::from_secs(1)).await;
                            count += 1;
                            // Since our task will be cancelled when the inbox is dropped
                            // we can just ignore the send error
                            tx.send(count).ok();
                        }
                    });
                    windows.push(MyWindow {
                        id: id_idx,
                        count: 0,
                        inbox,
                    });
                }

                windows.retain_mut(|MyWindow { id, count, inbox }| {
                    let mut open = true;
                    Window::new(format!("Window {id}"))
                        .open(&mut open)
                        .show(ui.ctx(), |ui| {
                            inbox.replace(ui, count);
                            ui.label(format!("Count: {count}"));
                        });
                    open
                });
            });
        },
    )
}
