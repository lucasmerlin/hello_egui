use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, Weak};

use egui::mutex::Mutex;
use egui::{Context, Id, Image, Sense, TextureHandle, Ui, Vec2, Widget};
use egui_inbox::UiInbox;
use serde::{Deserialize, Serialize};
use wry::raw_window_handle::HasWindowHandle;
use wry::{PageLoadEvent, WebView};

mod clip;
pub mod native_text_field;

use clip::{NativeClip, Shown};

pub struct EguiWebView {
    pub view: Arc<wry::WebView>,
    /// Only owned here: `webview_end_frame` reaches it through a weak ref.
    _clip: Arc<NativeClip>,
    id: Id,
    inbox: UiInbox<WebViewEvent>,
    current_image: Option<TextureHandle>,
    #[allow(dead_code)]
    context: Context,
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

pub enum WebViewEvent {
    ScreenshotReceived(TextureHandle),
    Focus,
    Blur,
    Loading(String),
    Loaded(String),
    Ipc(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum PageCommand {
    // Screenshot,
    Click { x: f32, y: f32 },
    Back,
    Forward,
}

pub struct WebViewResponse {
    pub events: Vec<WebViewEvent>,
    pub egui_response: egui::Response,
}

impl EguiWebView {
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        window: &impl HasWindowHandle,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Self {
        let (tx, inbox) = UiInbox::channel();
        let id = id.into();
        ctx.memory_mut(|mem| {
            mem.data
                .get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                )
                .clone()
        });

        let mut builder = wry::WebViewBuilder::new();

        builder = build(builder);

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
        let web_view = Arc::new(builder.build_as_child(window).unwrap());

        *view_ref.lock() = Some(web_view.clone());

        #[allow(clippy::arc_with_non_send_sync)]
        let clip = Arc::new(NativeClip::new(&web_view));

        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state
                .views
                .insert(id, (Arc::downgrade(&web_view), Arc::downgrade(&clip)));
        });

        Self {
            inbox,
            view: web_view,
            _clip: clip,
            id,
            current_image: None,
            context: ctx.clone(),
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

    #[allow(dead_code)]
    fn take_screenshot() {
        // let ctx = self.context.clone();
        // let tx = self.inbox.sender();

        // // TODO: This requires a screenshot feature in wry, https://github.com/tauri-apps/wry/pull/266
        // self.view
        //     .screenshot(wry::ScreenshotRegion::Visible, move |data| {
        //         let ctx = ctx.clone();
        //         let tx = tx.clone();
        //         if let Ok(screenshot) = data {
        //             let image = image::load_from_memory(&screenshot).unwrap();
        //
        //             let data = image.into_rgba8();
        //
        //             let handle = ctx.load_texture(
        //                 "browser_screenshot",
        //                 ColorImage::from_rgba_unmultiplied(
        //                     [data.width() as usize, data.height() as usize],
        //                     &data,
        //                 ),
        //                 Default::default(),
        //             );
        //             tx.send(WebViewEvent::ScreenshotReceived(handle)).ok();
        //         }
        //     })
        //     .ok();
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
        //self.take_screenshot();

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

        // The native view is placed in `webview_end_frame`, once the popups
        // that may cover it have been shown.
        let shown = Shown {
            layer: ui.layer_id(),
            rect: response.rect,
            clip: ui.clip_rect(),
        };
        ui.ctx().memory_mut(|mem| {
            let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.shown_this_frame.insert(self.id, shown);
        });

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

#[derive(Clone, Debug)]
struct GlobalWebViewState {
    views: HashMap<Id, (Weak<WebView>, Weak<NativeClip>)>,
    shown_this_frame: HashMap<Id, Shown>,
}

#[allow(unsafe_code)]
unsafe impl Send for GlobalWebViewState {}
#[allow(unsafe_code)]
unsafe impl Sync for GlobalWebViewState {}

pub const WEBVIEW_ID: &str = "egui_webview";

pub fn init_webview(ctx: &Context) {
    ctx.memory_mut(|mem| {
        if mem
            .data
            .get_temp::<GlobalWebViewState>(Id::new(WEBVIEW_ID))
            .is_some()
        {
            return;
        }
        mem.data.insert_temp(
            Id::new(WEBVIEW_ID),
            GlobalWebViewState {
                shown_this_frame: HashMap::new(),
                views: HashMap::new(),
            },
        );
    });
}

/// Place every webview and cut holes for the layers above it.
///
/// Call it once per frame, after all your UI, so every popup has been shown.
pub fn webview_end_frame(ctx: &Context) {
    let (views, shown) = ctx.memory_mut(|mem| {
        let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
            Id::new(WEBVIEW_ID),
            || unreachable!(),
        );
        state
            .views
            .retain(|_, (view, clip)| view.strong_count() > 0 && clip.strong_count() > 0);
        (
            state.views.clone(),
            std::mem::take(&mut state.shown_this_frame),
        )
    });

    // Outside the memory lock: the native calls can run webview callbacks.
    let placements = clip::placements(ctx, &shown);
    for (id, (view, clip)) in views {
        if let (Some(view), Some(clip)) = (view.upgrade(), clip.upgrade()) {
            clip.update(&view, placements.get(&id).and_then(Option::as_ref));
        }
    }
}
