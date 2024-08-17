use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, Weak};

use egui::mutex::Mutex;
use egui::{ColorImage, Context, Id, Image, Sense, TextureHandle, Ui, Vec2, Widget};
use egui_inbox::UiInbox;
use serde::{Deserialize, Serialize};
use wry::dpi::Size::Logical;
use wry::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Position, Size};
use wry::http::Request;
use wry::raw_window_handle::HasWindowHandle;
use wry::{PageLoadEvent, WebView};

pub mod native_text_field;

pub struct EguiWebView {
    pub view: Arc<wry::WebView>,
    id: Id,
    inbox: UiInbox<WebViewEvent>,
    current_image: Option<TextureHandle>,
    #[allow(dead_code)]
    context: Context,

    displayed_last_frame: bool,
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
    pub webview_visible: bool,
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

        let mut builder = wry::WebViewBuilder::new_as_child(window);

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
                                println!("Error loading webview script: {}", err);
                            };
                        }
                    }
                    PageLoadEvent::Finished => {}
                };

                tx_clone.send(WebViewEvent::Loaded(url)).ok();
            })
            .with_ipc_handler(move |msg| {
                let result = Self::handle_js_event(msg.into_body(), &ctx_clone);
                match result {
                    Ok(event) => {
                        tx.send(event).ok();
                    }
                    Err(err) => {
                        println!("Error handling js event: {}", err);
                    }
                }
            });

        #[allow(clippy::arc_with_non_send_sync)]
        let web_view = Arc::new(builder.build().unwrap());

        *view_ref.lock() = Some(web_view.clone());

        ctx.data_mut(|data| {
            let state = data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                Id::new(WEBVIEW_ID),
                || unreachable!(),
            );
            state.views.insert(id, Arc::downgrade(&web_view));
        });

        Self {
            inbox,
            view: web_view,
            id,
            current_image: None,
            context: ctx.clone(),
            displayed_last_frame: false,
        }
    }

    fn handle_js_event(msg: String, _ctx: &Context) -> Result<WebViewEvent, Box<dyn Error>> {
        let event = serde_json::from_str::<JsEvent>(&msg).map(|e| e.event);

        match event {
            Ok(JsEventType::Focus) => Ok(WebViewEvent::Focus),
            Ok(JsEventType::Blur) => Ok(WebViewEvent::Blur),
            Err(_) => Ok(WebViewEvent::Ipc(msg)),
        }
    }

    fn take_screenshot(&mut self) {
        let ctx = self.context.clone();
        let tx = self.inbox.sender();

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

    fn send_command(&self, command: PageCommand) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(&command)?;
        self.view
            .evaluate_script(&format!("__egui_webview_handle_command({})", json))?;
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
                WebViewEvent::Blur => {}
                WebViewEvent::Loaded(_) => {}
                WebViewEvent::Loading(_) => {}
                WebViewEvent::Ipc(_) => {}
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

        let my_layer = ui.layer_id();

        let is_my_layer_top =
            ui.memory(|mem| mem.areas().top_layer_id(my_layer.order) == Some(my_layer));

        if !is_my_layer_top {
            //response.surrender_focus();
        }

        if response.gained_focus() {
            println!("Gained focus");
            self.view.focus();
        }

        if let Some(image) = &self.current_image {
            Image::new(image).paint_at(ui, response.rect);
        }

        let should_display = ui.memory(|mem| (is_my_layer_top && !mem.any_popup_open()));

        dbg!(should_display);

        if !should_display && self.displayed_last_frame {
            self.current_image = None;
            self.take_screenshot();
        }
        self.displayed_last_frame = should_display;

        if should_display || self.current_image.is_none() {
            ui.ctx().memory_mut(|mem| {
                let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                );
                state.rendered_this_frame.insert(self.id);
            });
        }

        let wv_rect = response.rect * ui.ctx().zoom_factor();

        self.view
            .set_bounds(wry::Rect {
                position: Position::Logical(LogicalPosition::new(
                    wv_rect.min.x as f64,
                    wv_rect.min.y as f64,
                )),
                size: Size::Logical(LogicalSize::new(
                    wv_rect.width() as f64,
                    wv_rect.height() as f64,
                )),
            })
            .unwrap();

        WebViewResponse {
            events,
            egui_response: response,
            webview_visible: should_display,
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
    views: HashMap<Id, Weak<WebView>>,
    rendered_this_frame: HashSet<Id>,
}

unsafe impl Send for GlobalWebViewState {}
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
                    println!("Set visible false");
                    view.set_visible(false);
                } else {
                    println!("Set visible true");
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
