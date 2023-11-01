use eframe::egui;
use egui::{CentralPanel, Window};

use egui_suspense::EguiSuspense;

pub fn main() -> eframe::Result<()> {
    let mut suspenses = Vec::new();

    eframe::run_simple_native(
        "Suspense Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                Window::new("Main Window").show(ui.ctx(), |ui| {
                    ui.label("Hello World!");
                    if ui.button("Load").clicked() {
                        let suspense = EguiSuspense::reloadable(|cb| {
                            dbg!("Loading data...");
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                cb(if rand::random() {
                                    Ok("Hello".to_string())
                                } else {
                                    Err("OOPSIE WOOPSIE!".to_string())
                                });
                            });
                        })
                        .loading_ui(|ui| {
                            ui.label("Loading...");
                            ui.spinner();
                        })
                        .error_ui(|ui, error, state| {
                            ui.label(error);
                            if ui.button("Reload").clicked() {
                                state.reload();
                            }
                        });
                        suspenses.push(suspense);
                    }
                });

                suspenses.iter_mut().enumerate().for_each(|(i, suspense)| {
                    Window::new(i.to_string()).show(ui.ctx(), |ui| {
                        suspense.ui(ui, |ui, data, _state| {
                            ui.label(format!("Data: {:?}", data));
                        });
                    });
                });
            });
        },
    )
}
