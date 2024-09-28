use eframe::NativeOptions;
use egui::{CentralPanel, Window};

use egui_inbox::UiInbox;

struct MyWindow {
    id: usize,
    count: usize,
    inbox: UiInbox<usize>,
}

fn main() -> eframe::Result<()> {
    let mut id_idx = 0;
    let mut windows: Vec<MyWindow> = Vec::new();

    eframe::run_simple_native(
        "Inbox Example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                if ui.button("New Window").clicked() {
                    id_idx += 1;
                    let inbox = UiInbox::new();
                    let sender = inbox.sender();
                    windows.push(MyWindow {
                        id: id_idx,
                        count: 0,
                        inbox,
                    });

                    std::thread::spawn(move || {
                        let mut count = 0;
                        loop {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            count += 1;
                            if sender.send(count).is_err() {
                                break;
                            }
                        }
                        // You should see this after closing a window
                        println!("Stopped thread of window {id_idx}");
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
