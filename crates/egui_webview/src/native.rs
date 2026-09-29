use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;

use egui::mutex::Mutex;
use egui::{Context, Id, Image, Sense, TextureHandle, Ui, Vec2, Widget};
use egui_inbox::UiInbox;
use serde::{Deserialize, Serialize};
use wry::raw_window_handle::{HasWindowHandle, WindowHandle};
use wry::{PageLoadEvent, WebView};

use crate::clip::{NativeClip, Placement};
use crate::{show, with_state, WebViewEvent, WebViewResponse, WebViewSource};

#[derive(Debug)]
pub enum WebViewError {
    /// [`crate::set_parent_window`] wasn't called.
    NoParentWindow,
    Wry(wry::Error),
}

impl std::fmt::Display for WebViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoParentWindow => write!(f, "No parent window, call `set_parent_window` first"),
            Self::Wry(err) => write!(f, "Webview failed: {err}"),
        }
    }
}

impl Error for WebViewError {}

impl From<wry::Error> for WebViewError {
    fn from(err: wry::Error) -> Self {
        Self::Wry(err)
    }
}

/// Places the native view. Only [`EguiWebView`] owns it, and the global state holds a weak ref.
pub(crate) struct Placer {
    view: Arc<WebView>,
    clip: NativeClip,
}

impl Placer {
    pub fn place(&self, placement: Option<&Placement>) {
        self.clip.update(&self.view, placement);
    }
}

pub struct EguiWebView {
    pub view: Arc<wry::WebView>,
    placer: Arc<Placer>,
    id: Id,
    inbox: UiInbox<WebViewEvent>,
    current_image: Option<TextureHandle>,
}

impl Debug for EguiWebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiWebView").field("id", &self.id).finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct JsEvent {
    event: JsEventType,
    __egui_webview: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum JsEventType {
    Focus,
    Blur,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum PageCommand {
    Click { x: f32, y: f32 },
    Back,
    Forward,
}

impl EguiWebView {
    /// Create a webview in the window given to [`crate::set_parent_window`].
    pub fn from_source(
        ctx: &Context,
        id: impl Into<Id>,
        source: &WebViewSource,
    ) -> Result<Self, WebViewError> {
        let raw =
            with_state(ctx, |state| state.parent_window).ok_or(WebViewError::NoParentWindow)?;
        // SAFETY: `set_parent_window` requires the window to outlive every webview.
        #[allow(unsafe_code)]
        let window = unsafe { WindowHandle::borrow_raw(raw) };
        Self::try_new(ctx, id, &window, |builder| match source {
            WebViewSource::Html(html) => builder.with_html(html),
            WebViewSource::Url(url) => builder.with_url(url),
        })
    }

    /// # Panics
    /// If wry fails to create the webview.
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        window: &impl HasWindowHandle,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Self {
        Self::try_new(ctx, id, window, build).expect("Failed to create webview")
    }

    pub fn try_new(
        ctx: &Context,
        id: impl Into<Id>,
        window: &impl HasWindowHandle,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Result<Self, WebViewError> {
        crate::init_webview(ctx);
        let (tx, inbox) = UiInbox::channel();
        let id = id.into();

        let mut builder = build(wry::WebViewBuilder::new());

        #[allow(clippy::arc_with_non_send_sync)]
        let view_ref = Arc::new(Mutex::new(None::<Arc<WebView>>));
        let view_ref_weak = view_ref.clone();
        let ctx_clone = ctx.clone();

        let tx_clone = tx.clone();

        builder = builder
            .with_devtools(true)
            .with_on_page_load_handler(move |event, url| {
                match event {
                    PageLoadEvent::Started => {
                        let guard = view_ref_weak.lock();
                        if let Some(view) = guard.as_ref() {
                            if let Err(err) = view.evaluate_script(include_str!("webview.js")) {
                                println!("Error loading webview script: {err}");
                            }
                        }
                    }
                    PageLoadEvent::Finished => {}
                }

                tx_clone.send(WebViewEvent::Loaded(url)).ok();
            })
            .with_ipc_handler(move |msg: http::Request<String>| {
                let result = Self::handle_js_event(msg.body().clone(), &ctx_clone);
                tx.send(result).ok();
            });

        #[allow(clippy::arc_with_non_send_sync)]
        let web_view = Arc::new(builder.build_as_child(window)?);

        *view_ref.lock() = Some(web_view.clone());

        let render_state = with_state(ctx, |state| state.render_state.clone());
        #[allow(clippy::arc_with_non_send_sync)]
        let placer = Arc::new(Placer {
            clip: NativeClip::new(&web_view, id, render_state.as_ref()),
            view: web_view.clone(),
        });

        with_state(ctx, |state| {
            state.views.insert(id, Arc::downgrade(&placer));
        });

        Ok(Self {
            inbox,
            view: web_view,
            placer,
            id,
            current_image: None,
        })
    }

    /// Show another page.
    pub fn load(&self, source: &WebViewSource) -> Result<(), WebViewError> {
        match source {
            WebViewSource::Html(html) => self.view.load_html(html)?,
            WebViewSource::Url(url) => self.view.load_url(url)?,
        }
        Ok(())
    }

    fn handle_js_event(msg: String, _ctx: &Context) -> WebViewEvent {
        let event = serde_json::from_str::<JsEvent>(&msg).map(|e| e.event);

        match event {
            Ok(JsEventType::Focus) => WebViewEvent::Focus,
            Ok(JsEventType::Blur) => WebViewEvent::Blur,
            Err(_) => WebViewEvent::Ipc(msg),
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    fn send_command(&self, command: PageCommand) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(&command)?;
        self.view
            .evaluate_script(&format!("__egui_webview_handle_command({json})"))?;
        Ok(())
    }

    pub fn back(&self) {
        self.send_command(PageCommand::Back).ok();
    }

    pub fn forward(&self) {
        self.send_command(PageCommand::Forward).ok();
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) -> WebViewResponse {
        let response = ui.allocate_response(size, Sense::click());

        let events = self
            .inbox
            .read(ui)
            .inspect(|e| match e {
                WebViewEvent::ScreenshotReceived(img) => {
                    self.current_image = Some(img.clone());
                }
                WebViewEvent::Focus => {
                    ui.memory_mut(|mem| mem.request_focus(response.id));
                }
                _ => {}
            })
            .collect();

        if response.clicked() {
            response.request_focus();
            let pos = response.hover_pos();
            if let Some(pos) = pos {
                let relative = (pos - response.rect.min) / ui.ctx().pixels_per_point();

                self.send_command(PageCommand::Click {
                    x: relative.x,
                    y: relative.y,
                })
                .ok();
            }
        }

        if response.gained_focus() {
            self.view.focus().ok();
        }

        if let Some(image) = &self.current_image {
            Image::new(image).paint_at(ui, response.rect);
        }

        // The native view is placed at the end of the pass, once the popups
        // that may cover it have been shown.
        show(ui, self.id, response.rect, self.placer.clip.has_plane());

        WebViewResponse {
            events,
            egui_response: response,
        }
    }

    pub fn screenshot_ui(&mut self, ui: &mut Ui) {
        if let Some(img) = self.current_image.as_ref() {
            Image::new(img)
                .fit_to_exact_size(img.size_vec2() / ui.ctx().pixels_per_point())
                .ui(ui);
        }
    }
}
