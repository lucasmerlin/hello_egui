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
use crate::{
    show, with_state, WebViewBackend, WebViewEvent, WebViewOptions, WebViewResponse, WebViewSource,
};

#[derive(Debug)]
pub enum WebViewError {
    /// [`crate::set_parent_window`] wasn't called.
    NoParentWindow,
    Wry(wry::Error),
    /// Native webviews can't be embedded in a Wayland window. Use the CEF backend instead.
    Wayland,
    /// The CEF backend was asked for, but the `cef` feature is off.
    CefNotEnabled,
    Cef(String),
}

impl std::fmt::Display for WebViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoParentWindow => write!(f, "No parent window, call `set_parent_window` first"),
            Self::Wry(err) => write!(f, "Webview failed: {err}"),
            Self::Wayland => write!(
                f,
                "Native webviews don't work on Wayland: enable the `cef` feature, or run under X11"
            ),
            Self::CefNotEnabled => write!(f, "The CEF backend needs the `cef` feature"),
            Self::Cef(err) => write!(f, "CEF failed: {err}"),
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
    backend: Backend,
    id: Id,
    inbox: UiInbox<WebViewEvent>,
    current_image: Option<TextureHandle>,
}

enum Backend {
    Wry {
        view: Arc<wry::WebView>,
        placer: Arc<Placer>,
    },
    #[cfg(feature = "cef")]
    Cef(crate::cef::CefView),
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
    /// Create a webview, in the window given to [`crate::set_parent_window`] for a native one.
    ///
    /// The backend follows [`WebViewBackend::Auto`].
    pub fn from_source(
        ctx: &Context,
        id: impl Into<Id>,
        source: &WebViewSource,
    ) -> Result<Self, WebViewError> {
        Self::with_options(ctx, id, source, &WebViewOptions::default())
    }

    /// Create a webview with the given options.
    pub fn with_options(
        ctx: &Context,
        id: impl Into<Id>,
        source: &WebViewSource,
        options: &WebViewOptions,
    ) -> Result<Self, WebViewError> {
        crate::init_webview(ctx);
        let parent = with_state(ctx, |state| state.parent_window);
        match options.backend.resolve(parent) {
            WebViewBackend::Cef => Self::new_cef(ctx, id.into(), source),
            WebViewBackend::Native | WebViewBackend::Auto => {
                let raw = parent.ok_or(WebViewError::NoParentWindow)?;
                if matches!(raw, wry::raw_window_handle::RawWindowHandle::Wayland(_)) {
                    return Err(WebViewError::Wayland);
                }
                // SAFETY: `set_parent_window` requires the window to outlive every webview.
                #[allow(unsafe_code)]
                let window = unsafe { WindowHandle::borrow_raw(raw) };
                Self::try_new(ctx, id, &window, |builder| match source {
                    WebViewSource::Html(html) => builder.with_html(html),
                    WebViewSource::Url(url) => builder.with_url(url),
                })
            }
        }
    }

    #[cfg(feature = "cef")]
    fn new_cef(ctx: &Context, id: Id, source: &WebViewSource) -> Result<Self, WebViewError> {
        let (tx, inbox) = UiInbox::channel();
        Ok(Self {
            backend: Backend::Cef(crate::cef::CefView::new(ctx, source, tx)?),
            id,
            inbox,
            current_image: None,
        })
    }

    #[cfg(not(feature = "cef"))]
    fn new_cef(_ctx: &Context, _id: Id, _source: &WebViewSource) -> Result<Self, WebViewError> {
        Err(WebViewError::CefNotEnabled)
    }

    /// Create a native webview with wry, set up by `build`.
    ///
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
            backend: Backend::Wry {
                view: web_view,
                placer,
            },
            id,
            current_image: None,
        })
    }

    /// The wry webview, if this is a native one.
    pub fn wry_view(&self) -> Option<&Arc<wry::WebView>> {
        match &self.backend {
            Backend::Wry { view, .. } => Some(view),
            #[cfg(feature = "cef")]
            Backend::Cef(_) => None,
        }
    }

    /// Which backend shows this webview: [`WebViewBackend::Native`] or [`WebViewBackend::Cef`].
    pub fn backend(&self) -> WebViewBackend {
        match &self.backend {
            Backend::Wry { .. } => WebViewBackend::Native,
            #[cfg(feature = "cef")]
            Backend::Cef(_) => WebViewBackend::Cef,
        }
    }

    /// Show another page.
    pub fn load(&self, source: &WebViewSource) -> Result<(), WebViewError> {
        match &self.backend {
            Backend::Wry { view, .. } => match source {
                WebViewSource::Html(html) => view.load_html(html)?,
                WebViewSource::Url(url) => view.load_url(url)?,
            },
            #[cfg(feature = "cef")]
            Backend::Cef(view) => view.load(source),
        }
        Ok(())
    }

    /// Load the page at `url`.
    pub fn load_url(&self, url: &str) -> Result<(), WebViewError> {
        self.load(&WebViewSource::Url(url.to_owned()))
    }

    /// Run `script` in the page.
    pub fn evaluate_script(&self, script: &str) -> Result<(), WebViewError> {
        match &self.backend {
            Backend::Wry { view, .. } => view.evaluate_script(script)?,
            #[cfg(feature = "cef")]
            Backend::Cef(view) => view.evaluate_script(script),
        }
        Ok(())
    }

    pub fn reload(&self) {
        match &self.backend {
            Backend::Wry { view, .. } => {
                view.reload().ok();
            }
            #[cfg(feature = "cef")]
            Backend::Cef(view) => view.reload(),
        }
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
        self.evaluate_script(&format!("__egui_webview_handle_command({json})"))?;
        Ok(())
    }

    pub fn back(&self) {
        match &self.backend {
            Backend::Wry { .. } => {
                self.send_command(PageCommand::Back).ok();
            }
            #[cfg(feature = "cef")]
            Backend::Cef(view) => view.back(),
        }
    }

    pub fn forward(&self) {
        match &self.backend {
            Backend::Wry { .. } => {
                self.send_command(PageCommand::Forward).ok();
            }
            #[cfg(feature = "cef")]
            Backend::Cef(view) => view.forward(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) -> WebViewResponse {
        let (view, placer) = match &mut self.backend {
            Backend::Wry { view, placer } => (view.clone(), placer.clone()),
            #[cfg(feature = "cef")]
            Backend::Cef(view) => {
                let response = ui.allocate_response(size, Sense::click_and_drag());
                view.ui(ui, &response);
                return WebViewResponse {
                    events: self.inbox.read(ui).collect(),
                    egui_response: response,
                };
            }
        };

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
            view.focus().ok();
        }

        if let Some(image) = &self.current_image {
            Image::new(image).paint_at(ui, response.rect);
        }

        // The native view is placed at the end of the pass, once the popups
        // that may cover it have been shown.
        show(ui, self.id, response.rect, placer.clip.has_plane());

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
