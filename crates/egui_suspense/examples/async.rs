use eframe::NativeOptions;
use egui::CentralPanel;
use egui_suspense::EguiSuspense;
use futures::TryFutureExt;

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let mut timezones: EguiSuspense<Vec<String>, _> = EguiSuspense::reloadable_async(|| {
        reqwest::get("https://worldtimeapi.org/api/timezone").and_then(reqwest::Response::json)
    });

    let mut suspense = EguiSuspense::reloadable_async(|| {
        reqwest::get("https://worldtimeapi.org/api/ip").and_then(reqwest::Response::text)
    });

    eframe::run_ui_native(
        "Suspense Async Example",
        NativeOptions::default(),
        move |ui, _frame| {
            CentralPanel::default().show_inside(ui, |ui| {
                timezones.ui(ui, |ui, data, state| {
                    ui.label(format!("Timezones: {data:?}"));
                    if ui.button("Reload").clicked() {
                        state.reload();
                    }
                });

                suspense.ui(ui, |ui, data, state| {
                    if ui.button("Reload").clicked() {
                        state.reload();
                    }
                    ui.label(format!("Data: {data:?}"));
                });
            });
        },
    )
}
