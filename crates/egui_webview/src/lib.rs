use std::collections::HashMap;
use std::sync::Weak;

use egui::{Context, Id};

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

pub enum WebViewEvent {
    ScreenshotReceived(egui::TextureHandle),
    Focus,
    Blur,
    Loading(String),
    Loaded(String),
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
/// instead of cutting holes into the webview for them.
///
/// Popups then keep their shadows over the page, and a modal's backdrop dims it.
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
