use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::{Arc, Weak};

use egui::{Context, Id, Ui, Vec2};
use wry::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use wry::WebView;

pub mod native_text_field;

pub struct EguiWebView {
    view: Arc<wry::WebView>,
    id: Id,
}

impl Debug for EguiWebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiWebView").field("id", &self.id).finish()
    }
}

impl EguiWebView {
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Self {
        let id = id.into();
        let handle = ctx.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                )
                .clone()
        });

        let mut builder = wry::WebViewBuilder::new_as_child(&handle);

        builder = build(builder);

        #[allow(clippy::arc_with_non_send_sync)]
        let web_view = Arc::new(builder.build().unwrap());

        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.views.insert(id, Arc::downgrade(&web_view));
        });

        Self { view: web_view, id }
    }

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) {
        ui.ctx().memory_mut(|mem| {
            let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.rendered_this_frame.insert(self.id);
        });

        let (_id, rect) = ui.allocate_space(size);

        let rect = rect * ui.ctx().zoom_factor();

        self.view.set_bounds(wry::Rect {
            x: rect.min.x as i32,
            y: rect.min.y as i32,
            width: rect.width() as u32,
            height: rect.height() as u32,
        });
    }
}

#[derive(Clone, Debug)]
struct GlobalWebViewState {
    handle: RawWindowHandle,
    views: HashMap<Id, Weak<WebView>>,
    rendered_this_frame: HashSet<Id>,
}

unsafe impl HasRawWindowHandle for GlobalWebViewState {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.handle
    }
}

unsafe impl Send for GlobalWebViewState {}
unsafe impl Sync for GlobalWebViewState {}

pub const WEBVIEW_ID: &str = "egui_webview";

pub fn init_webview(ctx: &Context, handle: &impl HasRawWindowHandle) {
    let handle = handle.raw_window_handle();
    ctx.memory_mut(|mem| {
        mem.data.insert_temp(
            Id::new(WEBVIEW_ID),
            GlobalWebViewState {
                handle,
                rendered_this_frame: HashSet::new(),
                views: HashMap::new(),
            },
        )
    });
}

pub fn webview_end_frame(ctx: &Context) {
    ctx.memory_mut(|mem| {
        let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
            Id::new(WEBVIEW_ID),
            || unreachable!(),
        );
        state.views.retain(|id, view| {
            if let Some(view) = view.upgrade() {
                if !state.rendered_this_frame.contains(id) {
                    view.set_visible(false);
                } else {
                    view.set_visible(true);
                }

                true
            } else {
                false
            }
        });
        state.rendered_this_frame.clear();
    });
}
