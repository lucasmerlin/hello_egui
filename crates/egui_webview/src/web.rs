//! Webviews on the web: an `<iframe>` over egui's canvas.
//!
//! Each webview is a `<div>` next to egui's canvas, cropped to the visible part of its `Ui`.
//! It holds the `<iframe>`, and with paint planes a `<canvas>` above it that egui-wgpu
//! paints the layers above the webview into. The areas of those layers are cut out of
//! the iframe with a `clip-path`, so clicks in them reach egui's canvas.

use std::cell::RefCell;
use std::fmt::{Debug, Write as _};
use std::sync::Arc;

use egui::{Context, Id, Rect, Sense, Ui, Vec2};
use egui_inbox::UiInbox;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast as _;
use web_sys::{HtmlCanvasElement, HtmlElement, HtmlIFrameElement};

use crate::clip::{subtract, Placement};
use crate::{show, with_state, WebViewEvent, WebViewResponse, WebViewSource};

#[derive(Debug)]
pub enum WebViewError {
    /// [`crate::set_parent_canvas`] wasn't called.
    NoParentCanvas,
    Js(String),
}

impl std::fmt::Display for WebViewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoParentCanvas => write!(f, "No parent canvas, call `set_parent_canvas` first"),
            Self::Js(err) => write!(f, "Webview failed: {err}"),
        }
    }
}

impl std::error::Error for WebViewError {}

impl From<wasm_bindgen::JsValue> for WebViewError {
    fn from(err: wasm_bindgen::JsValue) -> Self {
        Self::Js(format!("{err:?}"))
    }
}

/// The canvas egui-wgpu paints the webview's paint plane into.
struct Plane {
    id: Id,
    canvas: HtmlCanvasElement,
    surfaces: egui_wgpu::PlaneSurfaces,
}

impl Plane {
    fn new(id: Id, render_state: &egui_wgpu::RenderState) -> Result<Option<Self>, WebViewError> {
        // A WebGL device can only paint into the canvas it was made for.
        if render_state.adapter.get_info().backend != egui_wgpu::wgpu::Backend::BrowserWebGpu {
            return Ok(None);
        }
        let canvas: HtmlCanvasElement = document()?.create_element("canvas")?.unchecked_into();
        set_style(
            &canvas,
            "position: absolute; left: 0; top: 0; pointer-events: none;",
        );
        let surface = match render_state
            .instance
            .create_surface(egui_wgpu::wgpu::SurfaceTarget::Canvas(canvas.clone()))
        {
            Ok(surface) => surface,
            Err(err) => {
                log(&format!("egui_webview: no paint plane surface: {err}"));
                return Ok(None);
            }
        };
        render_state.plane_surfaces.insert(id, surface);
        Ok(Some(Self {
            id,
            canvas,
            surfaces: render_state.plane_surfaces.clone(),
        }))
    }
}

impl Drop for Plane {
    fn drop(&mut self) {
        self.surfaces.remove(self.id);
        self.canvas.remove();
    }
}

/// Places the iframe. Only [`EguiWebView`] owns it, and the global state holds a weak ref.
pub(crate) struct Placer {
    parent_canvas: HtmlCanvasElement,
    container: HtmlElement,
    iframe: HtmlIFrameElement,
    plane: Option<Plane>,
    /// The CSS last applied, to skip style changes that change nothing.
    last: RefCell<Option<String>>,
    _on_load: Closure<dyn FnMut()>,
    on_message: Closure<dyn FnMut(web_sys::MessageEvent)>,
}

impl Placer {
    pub fn place(&self, placement: Option<&Placement>) {
        // Without a plane, hide while anything covers the page.
        let placement = placement.filter(|p| {
            p.visible.is_positive()
                && (self.plane.is_some() || (p.holes.is_empty() && !p.under_modal))
        });

        let css = placement.map(|placement| self.css(placement));
        let key = css.as_ref().map(|(a, b, c)| format!("{a}{b}{c}"));
        if *self.last.borrow() == key {
            return;
        }

        match &css {
            None => set_style(&self.container, "display: none;"),
            Some((container, iframe, plane)) => {
                set_style(&self.container, container);
                set_style(&self.iframe, iframe);
                if let Some(p) = &self.plane {
                    set_style(&p.canvas, plane);
                }
            }
        }
        self.last.replace(key);
    }

    /// The styles of the container, the iframe and the plane canvas.
    fn css(&self, placement: &Placement) -> (String, String, String) {
        let Placement {
            rect,
            visible,
            holes,
            under_modal,
            stack,
        } = placement;

        let left = f64::from(self.parent_canvas.offset_left()) + f64::from(visible.min.x);
        let top = f64::from(self.parent_canvas.offset_top()) + f64::from(visible.min.y);
        let container = format!(
            "position: absolute; left: {left}px; top: {top}px; width: {}px; height: {}px; \
             overflow: hidden; pointer-events: none; z-index: {};",
            visible.width(),
            visible.height(),
            1 + stack,
        );

        // In the iframe's own coordinates.
        let offset = rect.min - visible.min;
        let iframe_visible = Rect::from_min_size((-offset).to_pos2(), visible.size());
        let clip = if holes.is_empty() {
            "none".to_owned()
        } else {
            let holes: Vec<_> = holes
                .iter()
                .map(|hole| hole.translate(-rect.min.to_vec2()))
                .collect();
            let mut path = String::new();
            for part in subtract(iframe_visible, &holes) {
                write!(
                    path,
                    "M{} {}H{}V{}H{}Z",
                    part.min.x, part.min.y, part.max.x, part.max.y, part.min.x
                )
                .ok();
            }
            if path.is_empty() {
                "inset(50%)".to_owned()
            } else {
                format!("path('{path}')")
            }
        };
        // The modal takes all input.
        let pointer_events = if *under_modal { "none" } else { "auto" };
        let iframe = format!(
            "position: absolute; left: {}px; top: {}px; width: {}px; height: {}px; border: none; \
             clip-path: {clip}; pointer-events: {pointer_events};",
            offset.x,
            offset.y,
            rect.width(),
            rect.height(),
        );

        let plane = format!(
            "position: absolute; left: 0; top: 0; width: {}px; height: {}px; pointer-events: none;",
            visible.width(),
            visible.height(),
        );
        (container, iframe, plane)
    }
}

impl Drop for Placer {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window
                .remove_event_listener_with_callback(
                    "message",
                    self.on_message.as_ref().unchecked_ref(),
                )
                .ok();
        }
        self.plane = None;
        self.container.remove();
    }
}

pub struct EguiWebView {
    placer: Arc<Placer>,
    id: Id,
    inbox: UiInbox<WebViewEvent>,
}

impl Debug for EguiWebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiWebView").field("id", &self.id).finish()
    }
}

impl EguiWebView {
    /// Create a webview over the canvas given to [`crate::set_parent_canvas`].
    pub fn from_source(
        ctx: &Context,
        id: impl Into<Id>,
        source: &WebViewSource,
    ) -> Result<Self, WebViewError> {
        crate::init_webview(ctx);
        let id = id.into();
        let (parent_canvas, render_state) = with_state(ctx, |state| {
            (state.parent_canvas.clone(), state.render_state.clone())
        });
        let parent_canvas = parent_canvas.ok_or(WebViewError::NoParentCanvas)?;
        let parent = parent_canvas
            .parent_element()
            .ok_or_else(|| WebViewError::Js("The canvas has no parent element".to_owned()))?;

        let document = document()?;
        let container: HtmlElement = document.create_element("div")?.unchecked_into();
        set_style(&container, "display: none;");
        let iframe: HtmlIFrameElement = document.create_element("iframe")?.unchecked_into();
        container.append_child(&iframe)?;
        let plane = match &render_state {
            Some(render_state) => Plane::new(id, render_state)?,
            None => None,
        };
        if let Some(plane) = &plane {
            container.append_child(&plane.canvas)?;
        }
        parent.insert_before(&container, parent_canvas.next_sibling().as_ref())?;

        let (tx, inbox) = UiInbox::channel();
        let on_load = {
            let iframe = iframe.clone();
            let tx = tx.clone();
            Closure::<dyn FnMut()>::new(move || {
                tx.send(WebViewEvent::Loaded(iframe.src())).ok();
            })
        };
        iframe.set_onload(Some(on_load.as_ref().unchecked_ref()));

        // Every frame on the page can post to the window, so only take messages from our iframe.
        let on_message = {
            let iframe = iframe.clone();
            Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
                let from_page =
                    event
                        .source()
                        .zip(iframe.content_window())
                        .is_some_and(|(source, page)| {
                            wasm_bindgen::JsValue::from(source) == wasm_bindgen::JsValue::from(page)
                        });
                if let Some(text) = event.data().as_string().filter(|_| from_page) {
                    tx.send(WebViewEvent::Ipc(text)).ok();
                }
            })
        };
        web_sys::window()
            .ok_or_else(|| WebViewError::Js("No window".to_owned()))?
            .add_event_listener_with_callback("message", on_message.as_ref().unchecked_ref())?;

        #[allow(clippy::arc_with_non_send_sync)]
        let placer = Arc::new(Placer {
            parent_canvas,
            container,
            iframe,
            plane,
            last: RefCell::new(None),
            _on_load: on_load,
            on_message,
        });
        with_state(ctx, |state| {
            state.views.insert(id, Arc::downgrade(&placer));
        });

        let this = Self { placer, id, inbox };
        this.load(source)?;
        Ok(this)
    }

    /// Show another page.
    pub fn load(&self, source: &WebViewSource) -> Result<(), WebViewError> {
        let iframe = &self.placer.iframe;
        match source {
            WebViewSource::Html(html) => {
                // Without `allow-same-origin` the page gets an opaque origin.
                // The sandbox only applies from the next navigation on, so set it first.
                iframe.set_attribute("sandbox", "allow-scripts allow-forms allow-popups")?;
                iframe.set_srcdoc(html);
            }
            WebViewSource::Url(url) => {
                iframe.remove_attribute("sandbox")?;
                // `srcdoc` wins over `src`.
                iframe.remove_attribute("srcdoc")?;
                iframe.set_src(url);
            }
        }
        Ok(())
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) -> WebViewResponse {
        let response = ui.allocate_response(size, Sense::click());
        let events = self.inbox.read(ui).collect();

        // The iframe is placed at the end of the pass, once the popups
        // that may cover it have been shown.
        show(ui, self.id, response.rect, self.placer.plane.is_some());

        WebViewResponse {
            events,
            egui_response: response,
        }
    }
}

fn document() -> Result<web_sys::Document, WebViewError> {
    web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| WebViewError::Js("No document".to_owned()))
}

fn set_style(element: &web_sys::Element, css: &str) {
    element.set_attribute("style", css).ok();
}

fn log(msg: &str) {
    web_sys::console::warn_1(&msg.into());
}
