use egui::Ui;
use egui_hooks::UseHookExt as _;
use egui_infinite_scroll::InfiniteScroll;
use std::sync::Mutex;
use hello_egui_utils::run;

fn main() {
    run!(|ui| {
        my_standalone_ui(ui);
    })
}

fn my_standalone_ui(ui: &mut Ui) {
    let mut scroll = ui.use_state(
        || {
            Mutex::new(InfiniteScroll::default().end_loader(|cursor, callback| {
                let start = cursor.unwrap_or(0);
                let end = start + 100;
                callback(Ok(((start..end).collect(), Some(end))));
            }))
        },
        (),
    );

    scroll.lock().unwrap().ui(ui, 10, |ui, _index, item| {
        ui.label(format!("Item {}", item));
    });
}
