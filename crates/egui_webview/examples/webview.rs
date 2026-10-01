#![allow(clippy::needless_pass_by_value)] // It's ok here as it is an example
use eframe::{emath::Align, NativeOptions};
use egui::{CentralPanel, Context, Id, Layout, Popup, TextEdit, Widget, Window};

use egui_webview::{
    set_parent_window, use_paint_planes, EguiWebView, WebViewEvent, WebViewSource,
};

pub struct WebBrowser {
    id: Id,
    url_bar: String,
    view: EguiWebView,
}

impl WebBrowser {
    pub fn new(ctx: &Context, id: Id, url: &str) -> Self {
        // `EGUI_WEBVIEW_BACKEND=cef` (with the `cef` feature) renders it with CEF instead.
        let view = EguiWebView::from_source(ctx, id, &WebViewSource::Url(url.to_owned()))
            .expect("Failed to create webview");

        Self {
            id,
            url_bar: url.to_string(),
            view,
        }
    }

    pub fn ui(&mut self, ctx: &Context) -> bool {
        let mut open = true;
        Window::new(format!("Browser ({:?})", self.view.backend()))
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
                            self.view.load_url(&self.url_bar).unwrap();
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
    // CEF runs its helper processes from this executable.
    if let Some(code) = egui_webview::run_cef_subprocess() {
        std::process::exit(code);
    }

    let default_urls = [
        "https://www.rust-lang.org",
        "https://www.egui.rs",
        "https://www.reddit.com/r/rust",
        "https://www.github.com/lucasmerlin/hello_egui",
        "https://news.ycombinator.com",
    ];

    let mut windows = vec![];

    let mut count = 0;

    eframe::run_ui_native(
        "Dnd Example App",
        NativeOptions::default(),
        move |ui, frame| {
            egui_extras::install_image_loaders(ui.ctx());

            CentralPanel::default().show(ui, |ui| {
                if windows.is_empty() || ui.button("New Window").clicked() {
                    set_parent_window(ui.ctx(), frame).expect("No window handle");
                    if let Some(render_state) = frame.wgpu_render_state() {
                        use_paint_planes(ui.ctx(), render_state);
                    }

                    let url = default_urls[count % default_urls.len()];

                    windows.push(WebBrowser::new(
                        ui.ctx(),
                        Id::unique(format!("Window {count}")),
                        url,
                    ));
                    count += 1;
                }
            });

            windows.retain_mut(|w| w.ui(ui.ctx()));
        },
    )
}
