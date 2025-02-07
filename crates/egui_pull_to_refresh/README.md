# egui_pull_to_refresh

[![egui_ver](https://img.shields.io/badge/egui-0.31.0-blue)](https://github.com/emilk/egui)
[![Latest version](https://img.shields.io/crates/v/egui_pull_to_refresh.svg)](https://crates.io/crates/egui_pull_to_refresh)
[![Documentation](https://docs.rs/egui_pull_to_refresh/badge.svg)](https://docs.rs/egui_pull_to_refresh)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/egui_pull_to_refresh.svg)](https://crates.io/crates/egui_pull_to_refresh)



[content]:<>


... adds pull to refresh functionality to egui.
It works by wrapping a widget in a `PullToRefresh` widget, which will
display a refresh indicator when the user pulls down the widget.

## Demo Videos:

<https://github.com/lucasmerlin/hello_egui/assets/8009393/b8a7ca7f-4e4b-4ae9-bfad-1e98a88bf5ba>

<https://github.com/lucasmerlin/hello_egui/assets/8009393/c76e778e-6362-43cd-bef4-2d6e51eaf8d1>

## Usage
```rust
use egui::{Ui};
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
```

Have a look at the other [examples](https://github.com/lucasmerlin/hello_egui/tree/main/crates/egui_pull_to_refresh/examples) for more.
