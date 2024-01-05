use crate::EguiWebView;
use egui::{Context, Ui, Vec2};
use egui_inbox::UiInbox;

/// Since there still is no text input on when using egui/winit natively on ios and android
/// I thought that maybe we could use a webview as a very overcomplicated text input.
#[derive(Debug)]
pub struct NativeTextField {
    webview: EguiWebView,
    current_text: String,
    inbox: UiInbox<String>,
}

unsafe impl Send for NativeTextField {}
unsafe impl Sync for NativeTextField {}

impl NativeTextField {
    pub fn new(ctx: &Context) -> NativeTextField {
        let inbox = UiInbox::new();
        let inbox_clone = inbox.clone();
        let view = EguiWebView::new(ctx, "webview", |b| {
            b.with_html(include_str!("native_text_field.html"))
                .unwrap()
                .with_ipc_handler(move |msg| {
                    println!("Got message: {:?}", msg);
                    inbox_clone.send(msg);
                })
                .with_devtools(true)
        });
        NativeTextField {
            webview: view,
            inbox,
            current_text: "".to_string(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) {
        self.inbox.replace(ui, &mut self.current_text);
        self.webview.ui(ui, size);
    }

    pub fn current_text(&self) -> &str {
        &self.current_text
    }

    pub fn current_text_mut(&mut self, f: impl FnOnce(&mut String) -> bool) {
        if f(&mut self.current_text) {
            self.set_text(self.current_text.clone());
        }
    }

    pub fn set_text(&self, text: String) {
        self.webview
            .view
            .evaluate_script(&format!("set_text(\"{}\")", text.replace('\"', "\\\"")))
            .ok();
    }
}
