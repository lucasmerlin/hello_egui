use eframe::egui;
use egui::{Align2, Area, CentralPanel, Frame, Vec2};
use ehttp::Request;

use egui_inbox::{UiInbox, UiInboxSender};
use egui_pull_to_refresh::PullToRefresh;

pub fn main() -> eframe::Result<()> {
    let mut joke: Option<String> = None;
    let mut loading = true;
    let inbox = UiInbox::new();

    load_joke(inbox.sender());

    eframe::run_simple_native(
        "Pull to refresh dad jokes",
        Default::default(),
        move |ctx, _frame| {
            CentralPanel::default().show(ctx, |ui| {
                if let Some(j) = inbox.read(ui).last() {
                    joke = j;
                    loading = false;
                }

                let size = ui.available_size();

                let response = PullToRefresh::new(loading).ui(ui, |ui| {
                    // We have to set the height here so we can scroll everywhere on the screen
                    ui.set_height(size.y);

                    ui.vertical_centered(|ui| {
                        ui.allocate_space(Vec2::new(0.0, size.y / 4.0));
                        ui.label("Dad Jokes");
                        Frame::group(ui.style()).show(ui, |ui| {
                            ui.set_max_width(ui.available_width().min(400.0));
                            if let Some(joke) = joke.as_ref() {
                                ui.heading(joke);
                            }
                        });
                        ui.label("Pull to get a new joke!");
                        if ui.button("Or click me!").clicked() {
                            load_joke(inbox.sender());
                            loading = true;
                        }
                    });
                });

                if response.should_refresh() {
                    load_joke(inbox.sender());
                    loading = true;
                }
            });

            Area::new("attribution")
                .anchor(Align2::LEFT_BOTTOM, Vec2::new(8.0, -8.0))
                .show(ctx, |ui| {
                    ui.hyperlink_to(
                        "Jokes from icanhazdadjoke.com",
                        "https://icanhazdadjoke.com/",
                    );
                });
        },
    )
}

fn load_joke(tx: UiInboxSender<Option<String>>) {
    let mut request = Request::get("https://icanhazdadjoke.com/");
    request
        .headers
        .insert("Accept".to_string(), "text/plain".to_string());
    ehttp::fetch(request, move |response| {
        let response = response.unwrap();
        let joke = response.text().unwrap();
        tx.send(Some(joke.to_string())).ok();
    });
}
