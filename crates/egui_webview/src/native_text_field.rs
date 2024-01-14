use egui::{Context, Ui, Vec2};

use egui_inbox::UiInbox;

use crate::EguiWebView;

#[derive(Debug)]
pub enum TextFieldType {
    Text,
    Password,
    Email,
    Textarea,
}

impl TextFieldType {
    pub fn parameters(&self) -> &'static str {
        match self {
            TextFieldType::Text => "type=\"text\"",
            TextFieldType::Password => "type=\"password\"",
            TextFieldType::Email => "type=\"email\"",
            TextFieldType::Textarea => "",
        }
    }

    pub fn tag(&self) -> &'static str {
        match self {
            TextFieldType::Textarea => "textarea",
            _ => "input",
        }
    }
}

/// Since there still is no text input on when using egui/winit natively on ios and android
/// I thought that maybe we could use a webview as a very overcomplicated text input.
#[derive(Debug)]
pub struct NativeTextField {
    webview: EguiWebView,
    current_text: String,
    inbox: UiInbox<String>,
    field_type: TextFieldType,
}

unsafe impl Send for NativeTextField {}
unsafe impl Sync for NativeTextField {}

impl NativeTextField {
    pub fn new(ctx: &Context, text_field_type: TextFieldType) -> NativeTextField {
        let inbox = UiInbox::new();
        let tx = inbox.sender();
        let view = EguiWebView::new(ctx, "webview", |b| {
            b.with_html(
                include_str!("native_text_field.html")
                    .replace("_tag", text_field_type.tag())
                    .replace("_parameters", text_field_type.parameters()),
            )
            .unwrap()
            .with_ipc_handler(move |msg| {
                tx.send(msg).ok();
            })
            .with_devtools(true)
        });
        NativeTextField {
            webview: view,
            inbox,
            current_text: "".to_string(),
            field_type: text_field_type,
        }
    }

    pub fn field_type(&self) -> &TextFieldType {
        &self.field_type
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
