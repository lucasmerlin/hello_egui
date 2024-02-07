use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;

use eframe::egui;
use egui::{CentralPanel, Ui};

use egui_pull_to_refresh::PullToRefresh;

// This is the minimal example. Wrap some ui in a [`PullToRefresh`] widget
// and refresh when should_refresh() returns true.
fn my_ui(ui: &mut Ui, count: u64, loading: bool) -> bool {
    let response = PullToRefresh::new(loading).ui(ui, |ui| {
        ui.add_space(ui.available_size().y / 4.0);
        ui.vertical_centered(|ui| {
            ui.set_height(ui.available_size().y);
            ui.label("Pull to refresh demo");

            ui.label(format!("Count: {}", count));
        });
    });

    response.should_refresh()
}

#[derive(Debug, Clone)]
struct State {
    count: usize,
    loading: bool,
}

pub fn main() -> eframe::Result<()> {
    let state = Arc::new(Mutex::new(State {
        count: 0,
        loading: false,
    }));

    eframe::run_simple_native(
        "Pull to refresh minimal example",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                // Disable text selection, so it doesn't interfere with the drag gesture
                ui.style_mut().interaction.selectable_labels = false;
                ui.style_mut().interaction.multi_widget_text_select = false;
                let current_state = state.lock().unwrap().clone();

                let response = my_ui(ui, current_state.count as u64, current_state.loading);

                if response {
                    state.lock().unwrap().loading = true;
                    let state = state.clone();
                    thread::spawn(move || {
                        sleep(std::time::Duration::from_secs(1));
                        let mut state = state.lock().unwrap();
                        state.count += 1;
                        state.loading = false;
                    });
                }
            });
        },
    )
}
