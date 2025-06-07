use egui::{Context, CursorIcon, Id, Ui, Vec2};
use serde::{Deserialize, Serialize};
use wry::raw_window_handle::HasWindowHandle;

use crate::{EguiWebView, WebViewEvent};

#[derive(Debug)]
pub enum TextFieldType {
    Text,
    Password,
    Email,
    Textarea,
}

impl TextFieldType {
    #[must_use]
    pub fn parameters(&self) -> &'static str {
        match self {
            TextFieldType::Text => "type=\"text\"",
            TextFieldType::Password => "type=\"password\"",
            TextFieldType::Email => "type=\"email\"",
            TextFieldType::Textarea => "",
        }
    }

    #[must_use]
    pub fn tag(&self) -> &'static str {
        match self {
            TextFieldType::Textarea => "textarea",
            TextFieldType::Text | TextFieldType::Password | TextFieldType::Email => "input",
        }
    }
}

/// Since there still is no text input on when using egui/winit natively on ios and android
/// I thought that maybe we could use a webview as a very overcomplicated text input.
#[derive(Debug)]
pub struct NativeTextField {
    webview: EguiWebView,
    current_text: String,
    field_type: TextFieldType,
}

#[expect(unsafe_code, reason = "This is a webview, which is not Send/Sync")]
unsafe impl Send for NativeTextField {}
#[expect(unsafe_code, reason = "This is a webview, which is not Send/Sync")]
unsafe impl Sync for NativeTextField {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Event {
    Input { text: String },
    FocusOut,
}

impl NativeTextField {
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        text_field_type: TextFieldType,
        window: &impl HasWindowHandle,
    ) -> NativeTextField {
        let view = EguiWebView::new(ctx, id.into(), window, |b| {
            b.with_html(
                include_str!("native_text_field.html")
                    .replace("_tag", text_field_type.tag())
                    .replace("_parameters", text_field_type.parameters()),
            )
            .with_devtools(true)
        });
        NativeTextField {
            webview: view,
            current_text: String::new(),
            field_type: text_field_type,
        }
    }

    #[must_use]
    pub fn field_type(&self) -> &TextFieldType {
        &self.field_type
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) {
        let response = self.webview.ui(ui, size);
        response.events.into_iter().for_each(|e| {
            if let WebViewEvent::Ipc(text) = e {
                dbg!(&text);
                let event = serde_json::from_str::<Event>(&text);

                dbg!(&event);
                match event {
                    Ok(Event::Input { text }) => {
                        self.current_text = text;
                    }
                    Ok(Event::FocusOut) => {
                        println!("Focus out");
                        self.webview.view.set_visible(false).ok();
                        self.webview.view.set_visible(true).ok();
                        //response.egui_response.surrender_focus();
                        // let shift_key = ui.input(|i| i.modifiers.shift);
                        //response.egui_response.surrender_focus();
                        // ui.ctx().memory_mut(|mem| {
                        //     println!("Focus out 2");
                        //     // if shift_key {
                        //     //     mem.focus_item_in_direction(FocusDirection::Previous);
                        //     // } else {
                        //     //     mem.focus_item_in_direction(FocusDirection::Next);
                        //     // }
                        // })
                    }
                    Err(_) => {}
                }
            }
        });

        if response.egui_response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::Text);
        }
    }

    #[must_use]
    pub fn current_text(&self) -> &str {
        &self.current_text
    }

    pub fn current_text_mut(&mut self, f: impl FnOnce(&mut String) -> bool) {
        if f(&mut self.current_text) {
            self.set_text(&self.current_text);
        }
    }

    pub fn set_text(&self, text: &str) {
        self.webview
            .view
            .evaluate_script(&format!("set_text(\"{}\")", text.replace('\"', "\\\"")))
            .ok();
    }
}
