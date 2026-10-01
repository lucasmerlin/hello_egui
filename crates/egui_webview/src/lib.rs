use std::collections::HashMap;
use std::sync::Weak;

use egui::{Context, Id};

#[cfg(all(feature = "cef", not(target_arch = "wasm32")))]
mod cef;
mod clip;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_text_field;
#[cfg(target_arch = "wasm32")]
mod web;

use clip::Shown;
#[cfg(not(target_arch = "wasm32"))]
use native::Placer;
#[cfg(not(target_arch = "wasm32"))]
pub use native::{EguiWebView, WebViewError};
#[cfg(target_arch = "wasm32")]
use web::Placer;
#[cfg(target_arch = "wasm32")]
pub use web::{EguiWebView, WebViewError};

/// What a webview shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebViewSource {
    /// A page given as HTML.
    ///
    /// On the web, the page gets an opaque origin, so it can't reach the page that shows it.
    Html(String),

    /// A page to load from a URL.
    Url(String),
}

/// Which engine shows a webview.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WebViewBackend {
    /// The `EGUI_WEBVIEW_BACKEND` environment variable if it is set, to `native` or `cef`.
    /// Otherwise CEF on Wayland, where a native webview can't be embedded, and the native
    /// webview everywhere else.
    #[default]
    Auto,

    /// The platform's webview (WKWebView, WebView2 or WebKitGTK), as a native view over the
    /// window. Needs [`set_parent_window`]. Not available on Wayland.
    Native,

    /// Chromium, through CEF, rendered off screen into an egui texture. egui paints over it
    /// like over any image. Needs the `cef` feature, and [`run_cef_subprocess`] in `main`.
    Cef,
}

#[cfg(not(target_arch = "wasm32"))]
impl WebViewBackend {
    /// Turn [`Self::Auto`] into the backend to use for a webview in `parent`.
    fn resolve(self, parent: Option<wry::raw_window_handle::RawWindowHandle>) -> Self {
        if self != Self::Auto {
            return self;
        }
        match std::env::var("EGUI_WEBVIEW_BACKEND")
            .map(|value| value.to_ascii_lowercase())
            .as_deref()
        {
            Ok("native" | "wry") => return Self::Native,
            Ok("cef") => return Self::Cef,
            Ok(other) => eprintln!(
                "egui_webview: unknown EGUI_WEBVIEW_BACKEND {other:?}, expected `native` or `cef`"
            ),
            Err(_) => {}
        }
        let wayland = matches!(
            parent,
            Some(wry::raw_window_handle::RawWindowHandle::Wayland(_))
        );
        if wayland {
            Self::Cef
        } else {
            Self::Native
        }
    }
}

/// How to create a webview.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct WebViewOptions {
    pub backend: WebViewBackend,
}

#[cfg(not(target_arch = "wasm32"))]
impl WebViewOptions {
    #[must_use]
    pub fn backend(mut self, backend: WebViewBackend) -> Self {
        self.backend = backend;
        self
    }
}

/// Run a CEF helper process and return its exit code, if this process is one.
///
/// CEF starts its helper processes (renderer, GPU, …) from the app's own executable. Call
/// this first thing in `main`, and exit with the code it returns:
///
/// ```no_run
/// if let Some(code) = egui_webview::run_cef_subprocess() {
///     std::process::exit(code);
/// }
/// ```
///
/// Without the `cef` feature this does nothing and returns `None`.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_cef_subprocess() -> Option<i32> {
    #[cfg(feature = "cef")]
    {
        cef::run_subprocess()
    }
    #[cfg(not(feature = "cef"))]
    {
        None
    }
}

pub enum WebViewEvent {
    ScreenshotReceived(egui::TextureHandle),
    Focus,
    Blur,
    Loading(String),
    Loaded(String),

    /// A string the page sent.
    ///
    /// On native and with CEF the page sends it with `window.ipc.postMessage(text)`,
    /// on the web with `window.parent.postMessage(text, "*")`.
    Ipc(String),
}

pub struct WebViewResponse {
    pub events: Vec<WebViewEvent>,
    pub egui_response: egui::Response,
}

#[derive(Clone, Default)]
struct GlobalWebViewState {
    views: HashMap<Id, Weak<Placer>>,
    shown_this_frame: HashMap<Id, Shown>,
    /// Set by [`use_paint_planes`].
    render_state: Option<egui_wgpu::RenderState>,
    /// Set by [`set_parent_window`].
    #[cfg(not(target_arch = "wasm32"))]
    parent_window: Option<wry::raw_window_handle::RawWindowHandle>,
    /// Set by [`set_parent_canvas`].
    #[cfg(target_arch = "wasm32")]
    parent_canvas: Option<web_sys::HtmlCanvasElement>,
}

// Webviews and DOM elements are only touched from the thread that runs egui.
#[allow(unsafe_code)]
unsafe impl Send for GlobalWebViewState {}
#[allow(unsafe_code)]
unsafe impl Sync for GlobalWebViewState {}

pub const WEBVIEW_ID: &str = "egui_webview";

fn with_state<R>(ctx: &Context, f: impl FnOnce(&mut GlobalWebViewState) -> R) -> R {
    ctx.data_mut(|data| {
        f(data.get_temp_mut_or_default::<GlobalWebViewState>(Id::unique(WEBVIEW_ID)))
    })
}

/// Places every webview at the end of each pass, once all popups that may cover it are shown.
struct WebViewPlugin;

impl egui::Plugin for WebViewPlugin {
    fn debug_name(&self) -> &'static str {
        "egui_webview"
    }

    fn on_end_pass(&mut self, ui: &mut egui::Ui) {
        end_pass(ui.ctx());
    }
}

/// Set up webviews for this context. Calling it again does nothing.
pub fn init_webview(ctx: &Context) {
    with_state(ctx, |_| {});
    ctx.add_plugin(WebViewPlugin);
}

/// The native window that webviews are added to.
///
/// Needed before [`EguiWebView::from_source`] creates a webview.
/// The window has to outlive every webview.
#[cfg(not(target_arch = "wasm32"))]
pub fn set_parent_window(
    ctx: &Context,
    window: &impl wry::raw_window_handle::HasWindowHandle,
) -> Result<(), wry::raw_window_handle::HandleError> {
    let raw = window.window_handle()?.as_raw();
    init_webview(ctx);
    with_state(ctx, |state| state.parent_window = Some(raw));
    Ok(())
}

/// The canvas egui paints into. Webviews are placed over it.
///
/// Needed before [`EguiWebView::from_source`] creates a webview.
#[cfg(target_arch = "wasm32")]
pub fn set_parent_canvas(ctx: &Context, canvas: web_sys::HtmlCanvasElement) {
    init_webview(ctx);
    with_state(ctx, |state| state.parent_canvas = Some(canvas));
}

/// Paint the egui layers above each webview into a transparent surface over it,
/// so popups and windows show over the page.
///
/// Without it, a webview hides while anything covers it.
/// Needs a wgpu renderer, and applies to webviews created after this call.
/// Only macOS and the web (with WebGPU) have paint planes so far; elsewhere this changes nothing.
pub fn use_paint_planes(ctx: &Context, render_state: &egui_wgpu::RenderState) {
    init_webview(ctx);
    with_state(ctx, |state| state.render_state = Some(render_state.clone()));
}

/// Place every webview, and keep it out from under the layers above it.
fn end_pass(ctx: &Context) {
    let (views, shown) = with_state(ctx, |state| {
        state.views.retain(|_, placer| placer.strong_count() > 0);
        (
            state.views.clone(),
            std::mem::take(&mut state.shown_this_frame),
        )
    });

    // Outside the memory lock: the native calls can run webview callbacks.
    let placements = clip::placements(ctx, &shown);
    for (id, placer) in views {
        if let Some(placer) = placer.upgrade() {
            placer.place(placements.get(&id));
        }
    }
}

/// Note that the webview with this id is shown this pass, and add its paint plane.
fn show(ui: &egui::Ui, id: Id, rect: egui::Rect, has_plane: bool) {
    use egui::emath::GuiRounding as _;

    let shown = Shown {
        layer: ui.layer_id(),
        rect,
        visible: rect
            .intersect(ui.clip_rect())
            .round_to_pixels(ui.pixels_per_point()),
    };
    if has_plane && shown.visible.is_positive() {
        ui.ctx().add_paint_plane(id, shown.layer, shown.visible);
    }
    with_state(ui.ctx(), |state| {
        state.shown_this_frame.insert(id, shown);
    });
}
