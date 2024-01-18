use egui::{popup_above_or_below_widget, AboveOrBelow, Id, Window};
use egui_webview::{init_webview, webview_end_frame};
use wry::WebViewExtMacOS;

pub fn main() -> eframe::Result<()> {
    let mut view = None;

    let mut url_bar = "https://malmal.io".to_string();

    eframe::run_simple_native("Dnd Example App", Default::default(), move |ctx, _frame| {
        egui_extras::install_image_loaders(ctx);

        if view.is_none() {
            init_webview(ctx, _frame);

            let mut web_view =
                egui_webview::EguiWebView::new(ctx, "webview", |b| b.with_url(&url_bar).unwrap());

            let option = _frame.wgpu_render_state();
            if let Some(option) = option {
                web_view.set_wgpu_ctx(option.clone());
            }

            view = Some(web_view);
        }

        let view = view.as_mut().unwrap();

        Window::new("Hello").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Button icon arrow left
                if ui.button("◀").clicked() {
                    view.back();
                }

                if ui.button("▶").clicked() {
                    view.forward();
                }

                ui.label("URL:");
                let text_resp = ui.text_edit_singleline(&mut url_bar);
                let btn_resp = ui.button("Open");
                if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                    || btn_resp.clicked()
                {
                    view.view.load_url(&url_bar);
                };

                let popup_id = Id::new("Browser Menu");
                let mut menu_button = ui.button("☰");
                if menu_button.clicked() {
                    ui.memory_mut(|mem| {
                        mem.toggle_popup(popup_id);
                    });
                }

                // Hack to move the popup origin so it is aligned with the right side of the button.
                menu_button.rect = menu_button.rect.translate(egui::vec2(-200.0, 4.0));

                popup_above_or_below_widget(
                    ui,
                    popup_id,
                    &menu_button,
                    AboveOrBelow::Below,
                    |ui| {
                        ui.set_width(ui.min_size().x + 200.0);
                        ui.button("I have no function");
                        ui.button("My existence is meaningless");
                        ui.button("Why did you click me?");
                    },
                );
            });

            view.ui(ui, ui.available_size());
        });

        Window::new("Hello 2").show(ctx, |ui| {
            view.screenshot_ui(ui);
        });

        webview_end_frame(ctx);
    })
}
