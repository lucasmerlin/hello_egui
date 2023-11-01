use std::fmt::Debug;

use eframe::egui;
use egui::{CentralPanel, Frame, Window};

use egui_suspense::EguiSuspense;

// You can customize the loading and error uis by just placing a global function somewhere in your project.
fn custom_suspense<T: Debug + Send + Sync + 'static>(sus: EguiSuspense<T>) -> EguiSuspense<T> {
    sus.loading_ui(|ui| {
        ui.label("My custom loading ui!");
        ui.spinner();
    })
    .error_ui(|ui, error, state| {
        Frame::dark_canvas(ui.style()).show(ui, |ui| {
            ui.label("My custom error ui!");
            ui.label(error);
            if ui.button("My custom retry button").clicked() {
                state.reload();
            }
        });
    })
}

pub fn main() -> eframe::Result<()> {
    let mut suspenses = Vec::new();

    eframe::run_simple_native(
        "Custom Suspense Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                Window::new("Main Window").show(ui.ctx(), |ui| {
                    ui.label("Hello World!");
                    if ui.button("Load").clicked() {
                        // Then simply wrap the call where you create the suspense with your custom function.
                        let suspense = custom_suspense(EguiSuspense::reloadable(|cb| {
                            dbg!("Loading data...");
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                cb(if rand::random() {
                                    Ok("Hello".to_string())
                                } else {
                                    Err("OOPSIE WOOPSIE!".to_string())
                                });
                            });
                        }));
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
