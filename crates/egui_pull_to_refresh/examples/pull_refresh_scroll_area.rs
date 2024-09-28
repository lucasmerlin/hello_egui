use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;

use eframe::{egui, NativeOptions};
use egui::{CentralPanel, ScrollArea};

use egui_pull_to_refresh::PullToRefresh;

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

    let mut slider_value = 0.0;

    eframe::run_simple_native(
        "Pull to refresh minimal example",
        NativeOptions::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                // Disable text selection, so it doesn't interfere with the drag gesture
                ui.style_mut().interaction.selectable_labels = false;
                ui.style_mut().interaction.multi_widget_text_select = false;
                let current_state = state.lock().unwrap().clone();

                let response = PullToRefresh::new(current_state.loading).scroll_area_ui(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        ui.heading("Pull to refresh demo");
                        ui.label("It doesn't conflict with sliders!");
                        ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0).text("value"));

                        ui.label("And works with scroll areas!");
                        let count = current_state.count;
                        for i in count..count + 100 {
                            ui.label(format!("Hello {i}"));
                        }
                    })
                });

                if response.should_refresh() {
                    state.lock().unwrap().loading = true;
                    let state = state.clone();
                    thread::spawn(move || {
                        sleep(std::time::Duration::from_secs(1));
                        let mut state = state.lock().unwrap();
                        state.count += 100;
                        state.loading = false;
                    });
                }
            });
        },
    )
}
