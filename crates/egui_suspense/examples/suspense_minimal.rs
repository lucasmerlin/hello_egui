use eframe::egui;
use egui::CentralPanel;

use egui_suspense::EguiSuspense;

pub fn main() -> eframe::Result<()> {
    // The user will be able to reload the suspense by clicking the button.
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

    // This suspense cannot be retried / reloaded.
    let mut single_suspense = EguiSuspense::single_try(|cb| {
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(1));
            cb(if rand::random() {
                Ok("Hello".to_string())
            } else {
                Err("OOPSIE WOOPSIE!".to_string())
            });
        });
    });

    // You can also initialize a suspense with data already loaded.
    let mut already_loaded_suspense: EguiSuspense<String, String> =
        EguiSuspense::loaded("Hello".to_string());

    eframe::run_simple_native(
        "DnD Simple Example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                suspense.ui(ui, |ui, data, state| {
                    ui.label(format!("Data: {:?}", data));

                    if ui.button("Reload").clicked() {
                        state.reload();
                    }
                });

                ui.separator();

                single_suspense.ui(ui, |ui, data, _state| {
                    ui.label(format!("Data: {:?}", data));
                });

                ui.separator();

                already_loaded_suspense.ui(ui, |ui, data, _state| {
                    ui.label(format!("Data: {:?}", data));
                });
            });
        },
    )
}
