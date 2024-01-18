use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::Debug;
use std::sync::{Arc, Weak};

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use egui::load::SizedTexture;
use egui::mutex::Mutex;
use egui::{vec2, Context, Id, Image, Sense, Ui, Vec2, Widget};
use serde::{Deserialize, Serialize};
use wgpu::{Extent3d, TextureAspect, TextureDescriptor};
use wry::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use wry::{ScreenshotRegion, WebView};

use egui_inbox::UiInbox;

pub mod native_text_field;

pub struct EguiWebView {
    pub view: Arc<wry::WebView>,
    id: Id,
    inbox: UiInbox<WebViewEvent>,
    current_image: Option<TextureRef>,
    focused: bool,
    context: Context,

    wgpu_ctx: Option<egui_wgpu::RenderState>,
}

impl Debug for EguiWebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EguiWebView").field("id", &self.id).finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JsEvent {
    Screenshot { base64: String },
    Focus,
    Blur,
}

pub enum WebViewEvent {
    ScreenshotReceived(TextureRef),
    Focus,
    Blur,
    Loaded,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PageCommand {
    // Screenshot,
    Click { x: f32, y: f32 },
    Back,
    Forward,
}

enum TextureRef {
    Wgpu {
        sized_texture: SizedTexture,
        wgpu_ctx: egui_wgpu::RenderState,
    },
}

impl TextureRef {
    pub fn sized_texture(&self) -> &SizedTexture {
        match self {
            TextureRef::Wgpu { sized_texture, .. } => sized_texture,
        }
    }
}

impl Drop for TextureRef {
    fn drop(&mut self) {
        match self {
            TextureRef::Wgpu {
                sized_texture,
                wgpu_ctx,
            } => {
                wgpu_ctx.renderer.write().free_texture(&sized_texture.id);
            }
        }
    }
}

impl EguiWebView {
    pub fn new(
        ctx: &Context,
        id: impl Into<Id>,
        build: impl FnOnce(wry::WebViewBuilder) -> wry::WebViewBuilder,
    ) -> Self {
        let (tx, inbox) = UiInbox::channel();
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

        let view_ref = Arc::new(Mutex::new(None::<Arc<WebView>>));
        let view_ref_weak = view_ref.clone();
        let ctx_clone = ctx.clone();

        let tx_clone = tx.clone();

        builder = builder
            .with_devtools(true)
            .with_on_page_load_handler(move |event, url| {
                let arc = Some(view_ref_weak.clone());
                if let Some(guard) = arc {
                    let mut guard = guard.lock();
                    if let Some(view) = guard.as_ref() {
                        println!("Loading screenshot script");
                        view.evaluate_script(include_str!("html-to-image.js"));
                        view.evaluate_script(include_str!("screenshot.js"));
                    }
                }

                tx_clone.send(WebViewEvent::Loaded).ok();
            })
            .with_ipc_handler(move |msg| {
                let result = Self::handle_js_event(msg, &ctx_clone);
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
            focused: false,
            context: ctx.clone(),
            wgpu_ctx: None,
        }
    }

    pub fn set_wgpu_ctx(&mut self, ctx: egui_wgpu::RenderState) {
        self.wgpu_ctx = Some(ctx);
    }

    fn handle_js_event(msg: String, ctx: &Context) -> Result<WebViewEvent, Box<dyn Error>> {
        let event = serde_json::from_str(&msg)?;

        match event {
            JsEvent::Focus => Ok(WebViewEvent::Focus),
            JsEvent::Blur => Ok(WebViewEvent::Blur),
            JsEvent::Screenshot { base64 } => {
                let data = BASE64_STANDARD.decode(base64.as_bytes())?;
                let image = image::load_from_memory(&data)?;

                // Ok(WebViewEvent::ScreenshotReceived(ctx.load_texture(
                //     "browser_screenshot",
                //     ColorImage::from_rgba_unmultiplied(
                //         [image.width() as usize, image.height() as usize],
                //         &image.to_rgba8(),
                //     ),
                //     Default::default(),
                // )))
                Err("Not implemented".into())
            }
        }
    }

    fn take_screenshot(&mut self) {
        let ctx = self.context.clone();
        let tx = self.inbox.sender();

        let wgpu_ctx = self.wgpu_ctx.as_ref().map(|ctx| ctx.clone());

        self.view
            .screenshot_raw(ScreenshotRegion::Visible, move |data| {
                let ctx = ctx.clone();
                let tx = tx.clone();
                if let Ok(screenshot) = data {
                    // // let image = image::load_from_memory(&vec).unwrap();
                    // let image = ColorImage::from_rgba_premultiplied(
                    //     [screenshot.width as usize, screenshot.height as usize],
                    //     screenshot.data,
                    // );
                    // let handle = ctx.load_texture("browser_screenshot", image, Default::default());
                    // tx.send(WebViewEvent::ScreenshotReceived(handle)).ok();

                    let mut rgba8 = Vec::with_capacity(screenshot.data.len());

                    // convert BGRA to RGBA
                    for i in (0..screenshot.data.len()).step_by(4) {
                        rgba8.push(screenshot.data[i + 2]);
                        rgba8.push(screenshot.data[i + 1]);
                        rgba8.push(screenshot.data[i]);
                        rgba8.push(screenshot.data[i + 3]);
                    }

                    if let Some(wgpu_ctx) = &wgpu_ctx {
                        let size = Extent3d {
                            width: screenshot.width,
                            height: screenshot.height,
                            depth_or_array_layers: 1,
                        };
                        let texture = wgpu_ctx.device.create_texture(&TextureDescriptor {
                            label: Some("EguiWebView screenshot texture"),
                            size: size,
                            format: wgpu::TextureFormat::Rgba8UnormSrgb,
                            dimension: wgpu::TextureDimension::D2,
                            usage: wgpu::TextureUsages::COPY_DST
                                | wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::RENDER_ATTACHMENT,
                            mip_level_count: 1,
                            sample_count: 1,
                            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
                        });

                        wgpu_ctx.queue.write_texture(
                            wgpu::ImageCopyTexture {
                                texture: &texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: TextureAspect::All,
                            },
                            &rgba8,
                            wgpu::ImageDataLayout {
                                offset: 0,
                                bytes_per_row: Some(4 * screenshot.width),
                                rows_per_image: Some(screenshot.height),
                            },
                            size,
                        );

                        let id = wgpu_ctx.renderer.write().register_native_texture(
                            &wgpu_ctx.device,
                            &texture.create_view(&wgpu::TextureViewDescriptor {
                                label: Some("EguiWebView screenshot texture view"),
                                mip_level_count: None,
                                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                                dimension: Some(wgpu::TextureViewDimension::D2),
                                aspect: TextureAspect::All,
                                array_layer_count: None,
                                ..Default::default()
                            }),
                            wgpu::FilterMode::Linear,
                        );

                        tx.send(WebViewEvent::ScreenshotReceived(TextureRef::Wgpu {
                            sized_texture: SizedTexture::new(
                                id,
                                vec2(screenshot.width as f32, screenshot.height as f32),
                            ),
                            wgpu_ctx: wgpu_ctx.clone(),
                        }))
                        .ok();
                    }
                }
            })
            .ok();
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

    pub fn ui(&mut self, ui: &mut Ui, size: Vec2) {
        //self.take_screenshot();

        let response = ui.allocate_response(size, Sense::click());

        if let Some(img) = self.inbox.read(ui).last() {
            match img {
                WebViewEvent::ScreenshotReceived(img) => {
                    self.current_image = Some(img);
                    self.take_screenshot();
                }
                WebViewEvent::Focus => {
                    ui.memory_mut(|mem| mem.request_focus(response.id));
                }
                WebViewEvent::Blur => {}
                WebViewEvent::Loaded => {
                    self.take_screenshot();
                }
            }
        }

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
            ui.memory(|mut mem| mem.areas().top_layer_id(my_layer.order) == Some(my_layer));

        if !is_my_layer_top {
            response.surrender_focus();
        }

        if response.gained_focus() {
            println!("Gained focus");
            self.view.focus();
        }

        if response.lost_focus() {
            //self.take_screenshot();
        }

        println!("Focused: {}", response.has_focus());

        if let Some(image) = &self.current_image {
            Image::new(image.sized_texture().clone()).paint_at(ui, response.rect);
        }

        let should_display = ui
            .memory(|mem| mem.has_focus(response.id) || (is_my_layer_top && !mem.any_popup_open()));

        if should_display {
            ui.ctx().memory_mut(|mem| {
                let state = mem.data.get_temp_mut_or_insert_with::<GlobalWebViewState>(
                    Id::new(WEBVIEW_ID),
                    || unreachable!(),
                );
                state.rendered_this_frame.insert(self.id);
            });
        }

        let wv_rect = response.rect * ui.ctx().zoom_factor();

        self.view.set_bounds(wry::Rect {
            x: wv_rect.min.x as i32,
            y: wv_rect.min.y as i32,
            width: wv_rect.width() as u32,
            height: wv_rect.height() as u32,
        });
    }

    pub fn screenshot_ui(&mut self, ui: &mut Ui) {
        if let Some(img) = self.current_image.as_ref() {
            let source = img.sized_texture().clone();
            let sie = source.size;
            Image::new(source)
                .fit_to_exact_size(sie / ui.ctx().pixels_per_point())
                .ui(ui);
        }
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
