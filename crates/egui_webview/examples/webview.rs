use eframe::emath::Align;
use egui::{
    popup_above_or_below_widget, AboveOrBelow, CentralPanel, Context, Id, Layout, TextEdit, Widget,
    Window,
};
use wry::raw_window_handle::HasWindowHandle;

use egui_webview::{init_webview, webview_end_frame, EguiWebView, WebViewEvent};

pub struct WebBrowser {
    id: Id,
    url_bar: String,
    view: EguiWebView,
}

impl WebBrowser {
    pub fn new(ctx: Context, id: Id, url: &str, window: &impl HasWindowHandle) -> Self {
        let view = EguiWebView::new(&ctx, id, window, |b| b.with_url(url).unwrap());

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
                        let popup_id = Id::new("Browser Menu").with(self.id);
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
                                let _ = ui.button("I have no function");
                                let _ = ui.button("My existence is meaningless");
                                if ui.button("Why did you click me?").clicked() {
                                    self.view
                                        .view
                                        .load_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                                }
                            },
                        );

                        let btn_resp = ui.button("Open");
                        let text_resp = TextEdit::singleline(&mut self.url_bar)
                            .desired_width(ui.available_width())
                            .ui(ui);

                        if text_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            || btn_resp.clicked()
                        {
                            self.view.view.load_url(&self.url_bar);
                        };
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

    eframe::run_simple_native("Dnd Example App", Default::default(), move |ctx, _frame| {
        egui_extras::install_image_loaders(ctx);

        CentralPanel::default().show(ctx, |ui| {
            if windows.is_empty() || ui.button("New Window").clicked() {
                init_webview(ctx);

                let url = default_urls[count % default_urls.len()];

                windows.push(WebBrowser::new(
                    ctx.clone(),
                    Id::new(format!("Window {}", count)),
                    url,
                    _frame,
                ));
                count += 1;
            }
        });

        windows.retain_mut(|w| w.ui(ctx));

        webview_end_frame(ctx);
    })
}
