use eframe::{emath::Align, NativeOptions};
use egui::{CentralPanel, Context, Id, Layout, Popup, TextEdit, Widget, Window};
use wry::raw_window_handle::HasWindowHandle;

use egui_webview::{init_webview, webview_end_frame, EguiWebView, WebViewEvent};

pub struct WebBrowser {
    id: Id,
    url_bar: String,
    view: EguiWebView,
}

impl WebBrowser {
    pub fn new(ctx: &Context, id: Id, url: &str, window: &impl HasWindowHandle) -> Self {
        let view = EguiWebView::new(ctx, id, window, |b| b.with_url(url));

        Self {
            id,
            url_bar: url.to_string(),
            view,
        }
    }

    pub fn ui(&mut self, ctx: &Context) -> bool {
        let mut open = true;
        Window::new("Browser")
            .id(self.id)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Button icon arrow left
                    if ui.button("◀").clicked() {
                        self.view.back();
                    }

                    if ui.button("▶").clicked() {
                        self.view.forward();
                    }
                    ui.label("URL:");

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let menu_button = ui.button("☰");

                        Popup::menu(&menu_button).show(|ui| {
                            ui.set_width(ui.min_size().x + 200.0);
                            let _ = ui.button("I have no function");
                            let _ = ui.button("My existence is meaningless");
                            if ui.button("Why did you click me?").clicked() {
                                self.view
                                    .view
                                    .load_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                                    .unwrap();
                            }
                        });

                        let btn_resp = ui.button("Open");
                        let text_resp = TextEdit::singleline(&mut self.url_bar)
                            .desired_width(ui.available_width())
                            .ui(ui);

                        if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            || btn_resp.clicked()
                        {
                            self.view.view.load_url(&self.url_bar).unwrap();
                        }
                    });
                });

                self.view
                    .ui(ui, ui.available_size())
                    .events
                    .into_iter()
                    .for_each(|e| {
                        if let WebViewEvent::Loaded(url) = e {
                            self.url_bar = url;
                        }
                    });
            });
        open
    }
}

pub fn main() -> eframe::Result<()> {
    let default_urls = [
        "https://www.rust-lang.org",
        "https://www.egui.rs",
        "https://www.reddit.com/r/rust",
        "https://www.github.com/lucasmerlin/hello_egui",
        "https://news.ycombinator.com",
    ];

    let mut windows = vec![];

    let mut count = 0;

    eframe::run_simple_native(
        "Dnd Example App",
        NativeOptions::default(),
        move |ctx, frame| {
            egui_extras::install_image_loaders(ctx);

            CentralPanel::default().show(ctx, |ui| {
                if windows.is_empty() || ui.button("New Window").clicked() {
                    init_webview(ctx);

                    let url = default_urls[count % default_urls.len()];

                    windows.push(WebBrowser::new(
                        ctx,
                        Id::new(format!("Window {count}")),
                        url,
                        frame,
                    ));
                    count += 1;
                }
            });

            windows.retain_mut(|w| w.ui(ctx));

            webview_end_frame(ctx);
        },
    )
}
