//! Webviews that CEF (Chromium) renders off screen, for where a native webview can't be
//! embedded in the window, like on Wayland.
//!
//! Each webview is a windowless CEF browser. It paints into a pixel buffer that becomes an
//! egui texture, and the webview's widget forwards egui's input to it. There is no native
//! view, so egui paints over the page like over any image, and no holes or planes are needed.
//!
//! CEF starts its helper processes from the app's own binary, see
//! [`crate::run_cef_subprocess`].

// The code the cef-rs `wrap_*` macros expand to.
#![allow(clippy::transmute_ptr_to_ptr)]

mod keys;

use std::cell::{Cell, RefCell};
use std::fmt::Write as _;
use std::rc::{Rc, Weak};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

// The `wrap_*` macros call `add_ref` from this trait.
use cef::rc::Rc as _;
use cef::{
    sys, Browser, BrowserSettings, CefString, ImplBrowser, ImplBrowserHost, ImplCommandLine,
    ImplFrame, ImplMenuModel, KeyEvent, MouseEvent, WindowInfo,
};
use cef::{
    wrap_app, wrap_browser_process_handler, wrap_client, wrap_context_menu_handler,
    wrap_display_handler, wrap_life_span_handler, wrap_load_handler, wrap_render_handler,
    wrap_render_process_handler, App, BrowserProcessHandler, Client, ContextMenuHandler,
    DisplayHandler, ImplApp, ImplBrowserProcessHandler, ImplClient, ImplContextMenuHandler,
    ImplDisplayHandler, ImplLifeSpanHandler, ImplLoadHandler, ImplRenderHandler,
    ImplRenderProcessHandler, LifeSpanHandler, LoadHandler, RenderHandler, RenderProcessHandler,
    WrapApp, WrapBrowserProcessHandler, WrapClient, WrapContextMenuHandler, WrapDisplayHandler,
    WrapLifeSpanHandler, WrapLoadHandler, WrapRenderHandler, WrapRenderProcessHandler,
};
use egui::{
    pos2, Color32, ColorImage, Context, CursorIcon, Event, EventFilter, Modifiers, PointerButton,
    Pos2, Rect, Response, TextureHandle, TextureOptions, Ui, Vec2,
};
use egui_inbox::UiInboxSender;

use crate::{WebViewError, WebViewEvent, WebViewSource};

/// What a page posts with `window.ipc.postMessage` comes to the browser process as a console
/// message with this prefix. CEF has no other way to send a string from the page without
/// renderer-side message routing.
const IPC_PREFIX: &str = "\u{0}egui_webview_ipc:";

/// Run in each page before its own scripts, by the render process.
const IPC_SCRIPT: &str = r#"(() => {
    const log = console.log.bind(console);
    Object.defineProperty(window, "ipc", {
        value: Object.freeze({ postMessage: (message) => log("\u0000egui_webview_ipc:" + String(message)) }),
    });
})();"#;

/// CEF asks for message loop work when it needs it, but it may also need work it didn't ask
/// for, so the loop runs at least this often while a CEF webview exists.
const MAX_PUMP_DELAY: Duration = Duration::from_millis(100);

/// Points to scroll per line of a mouse wheel.
const POINTS_PER_LINE: f32 = 40.0;

static INIT: OnceLock<Result<(), String>> = OnceLock::new();

/// The contexts to repaint when CEF asks for message loop work.
static CONTEXTS: Mutex<Vec<Context>> = Mutex::new(Vec::new());

thread_local! {
    /// Every webview, to hide the ones not shown in a pass.
    static VIEWS: RefCell<Vec<Weak<Handle>>> = const { RefCell::new(Vec::new()) };
}

/// Run a CEF helper process, if this is one. See [`crate::run_cef_subprocess`].
pub fn run_subprocess() -> Option<i32> {
    if !is_subprocess() {
        return None;
    }
    if let Err(err) = load_library() {
        eprintln!("egui_webview: {err}");
        return Some(1);
    }
    let _ = cef::api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = cef::args::Args::new();
    let mut app = EguiApp::new();
    Some(cef::execute_process(
        Some(args.as_main_args()),
        Some(&mut app),
        std::ptr::null_mut(),
    ))
}

fn is_subprocess() -> bool {
    std::env::args().any(|arg| arg.starts_with("--type="))
}

/// Load the CEF framework on macOS: from the app bundle if there is one, otherwise from where
/// the `cef` crate put it at build time.
#[cfg(target_os = "macos")]
fn load_library() -> Result<(), String> {
    use std::os::unix::ffi::OsStrExt as _;

    const FRAMEWORK: &str = "Chromium Embedded Framework.framework/Chromium Embedded Framework";
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    let exe_dir = exe.parent().ok_or("no executable directory")?;
    // The main app in `Contents/MacOS`, or a helper app in `Contents/Frameworks/X Helper.app/Contents/MacOS`.
    let bundled = [exe_dir.join("../Frameworks"), exe_dir.join("../../..")]
        .into_iter()
        .map(|dir| dir.join(FRAMEWORK))
        .find(|path| path.exists());
    let path = bundled
        .or_else(|| sys::get_cef_dir().map(|dir| dir.join(FRAMEWORK)))
        .ok_or("CEF framework not found")?;
    let path = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(|err| err.to_string())?;
    // SAFETY: a valid C string, which outlives the call.
    #[allow(unsafe_code)]
    let name = unsafe { &*path.as_ptr() };
    if cef::load_library(Some(name)) == 1 {
        Ok(())
    } else {
        Err(format!("Failed to load the CEF framework from {path:?}"))
    }
}

#[cfg(not(target_os = "macos"))]
#[expect(clippy::unnecessary_wraps)]
fn load_library() -> Result<(), String> {
    // Linked at build time.
    Ok(())
}

/// Start CEF in this process, once.
fn initialize(ctx: &Context) -> Result<(), WebViewError> {
    let result = INIT.get_or_init(|| {
        if is_subprocess() {
            return Err(
                "This is a CEF helper process: call `egui_webview::run_cef_subprocess` first in `main`"
                    .to_owned(),
            );
        }
        load_library()?;
        let _ = cef::api_hash(sys::CEF_API_VERSION_LAST, 0);

        let exe = std::env::current_exe().map_err(|err| err.to_string())?;
        let mut settings = cef::Settings {
            windowless_rendering_enabled: 1,
            external_message_pump: 1,
            no_sandbox: 1,
            browser_subprocess_path: exe.to_string_lossy().as_ref().into(),
            // A cache of its own for each process, so two instances of an app don't fight over it.
            root_cache_path: std::env::temp_dir()
                .join(format!("egui_webview_cef_{}", std::process::id()))
                .to_string_lossy()
                .as_ref()
                .into(),
            log_severity: sys::cef_log_severity_t::LOGSEVERITY_WARNING.into(),
            ..Default::default()
        };
        resource_paths(&exe, &mut settings);

        // CEF keeps pointers into the arguments.
        let args = Box::leak(Box::new(cef::args::Args::new()));
        let mut app = EguiApp::new();
        if cef::initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut app),
            std::ptr::null_mut(),
        ) == 1
        {
            Ok(())
        } else {
            Err("CEF failed to initialize".to_owned())
        }
    });
    result.clone().map_err(WebViewError::Cef)?;

    let mut contexts = CONTEXTS.lock().expect("poisoned");
    if !contexts.iter().any(|c| c == ctx) {
        contexts.push(ctx.clone());
        ctx.add_plugin(CefPlugin);
    }
    Ok(())
}

/// Point CEF at its resources, if they aren't next to the executable, where a packaged app
/// has them.
fn resource_paths(exe: &std::path::Path, settings: &mut cef::Settings) {
    let Some(cef_dir) = sys::get_cef_dir() else {
        return;
    };
    #[cfg(target_os = "macos")]
    {
        let in_bundle = exe
            .parent()
            .is_some_and(|dir| dir.join("../Frameworks").exists());
        if !in_bundle {
            settings.framework_dir_path = cef_dir
                .join("Chromium Embedded Framework.framework")
                .to_string_lossy()
                .as_ref()
                .into();
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let next_to_exe = exe
            .parent()
            .is_some_and(|dir| dir.join("icudtl.dat").exists());
        if !next_to_exe {
            settings.resources_dir_path = cef_dir.to_string_lossy().as_ref().into();
            settings.locales_dir_path = cef_dir.join("locales").to_string_lossy().as_ref().into();
        }
    }
}

/// Runs CEF's message loop work, and hides the webviews not shown in the last pass.
struct CefPlugin;

impl egui::Plugin for CefPlugin {
    fn debug_name(&self) -> &'static str {
        "egui_webview_cef"
    }

    fn on_begin_pass(&mut self, ui: &mut Ui) {
        cef::do_message_loop_work();

        let pass = ui.ctx().cumulative_pass_nr();
        let alive = VIEWS.with_borrow_mut(|views| {
            views.retain(|view| view.strong_count() > 0);
            for view in views.iter().filter_map(Weak::upgrade) {
                let shown = view.last_shown.get() + 1 >= pass;
                if !shown && !view.hidden.get() {
                    if let Some(host) = view.browser.host() {
                        host.was_hidden(1);
                    }
                    view.hidden.set(true);
                }
            }
            !views.is_empty()
        });
        if alive {
            ui.ctx().request_repaint_after(MAX_PUMP_DELAY);
        }
    }
}

fn wake(delay: Duration) {
    for ctx in CONTEXTS.lock().expect("poisoned").iter() {
        ctx.request_repaint_after(delay);
    }
}

wrap_app! {
    struct EguiApp {}

    impl App {
        fn on_before_command_line_processing(
            &self,
            process_type: Option<&CefString>,
            command_line: Option<&mut cef::CommandLine>,
        ) {
            let Some(command_line) = command_line else {
                return;
            };
            let is_browser = process_type.is_none_or(|t| t.to_string().is_empty());
            if !is_browser {
                return;
            }
            // Don't ask for the macOS keychain, which shows a prompt.
            command_line.append_switch(Some(&"use-mock-keychain".into()));
            #[cfg(target_os = "linux")]
            if command_line.has_switch(Some(&"ozone-platform".into())) == 0 {
                let platform = if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                    "wayland"
                } else {
                    "x11"
                };
                command_line.append_switch_with_value(
                    Some(&"ozone-platform".into()),
                    Some(&platform.into()),
                );
            }
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(EguiBrowserProcessHandler::new())
        }

        fn render_process_handler(&self) -> Option<RenderProcessHandler> {
            Some(EguiRenderProcessHandler::new())
        }
    }
}

wrap_browser_process_handler! {
    struct EguiBrowserProcessHandler {}

    impl BrowserProcessHandler {
        fn on_schedule_message_pump_work(&self, delay_ms: i64) {
            wake(Duration::from_millis(delay_ms.max(0).unsigned_abs()));
        }
    }
}

wrap_render_process_handler! {
    struct EguiRenderProcessHandler {}

    impl RenderProcessHandler {
        fn on_context_created(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut cef::Frame>,
            _context: Option<&mut cef::V8Context>,
        ) {
            if let Some(frame) = frame {
                frame.execute_java_script(Some(&IPC_SCRIPT.into()), None, 0);
            }
        }
    }
}

/// One frame of a browser or of its popup, in physical pixels.
#[derive(Default)]
struct Layer {
    size: [usize; 2],
    bgra: Vec<u8>,
    /// The part that changed since the last upload: `[x, y, width, height]`.
    dirty: Option<[usize; 4]>,
    /// The size changed, so the whole texture has to be replaced.
    resized: bool,
}

impl Layer {
    fn paint(&mut self, buffer: &[u8], size: [usize; 2], dirty: &[cef::Rect]) {
        if size != self.size {
            self.size = size;
            self.bgra = buffer.to_vec();
            self.resized = true;
            self.dirty = Some([0, 0, size[0], size[1]]);
            return;
        }
        let stride = size[0] * 4;
        for rect in dirty {
            let [x, y, w, h] = clamp_rect(rect, size);
            for row in y..y + h {
                let range = row * stride + x * 4..row * stride + (x + w) * 4;
                self.bgra[range.clone()].copy_from_slice(&buffer[range]);
            }
            self.dirty = Some(match self.dirty {
                None => [x, y, w, h],
                Some([dx, dy, dw, dh]) => {
                    let (x0, y0) = (dx.min(x), dy.min(y));
                    let (x1, y1) = ((dx + dw).max(x + w), (dy + dh).max(y + h));
                    [x0, y0, x1 - x0, y1 - y0]
                }
            });
        }
    }

    /// Bring `texture` up to date with this layer.
    fn upload(&mut self, ctx: &Context, name: &str, texture: &mut Option<TextureHandle>) {
        let Some([x, y, w, h]) = self.dirty.take() else {
            return;
        };
        let stride = self.size[0] * 4;
        let image = |x: usize, y: usize, w: usize, h: usize| {
            let mut pixels = Vec::with_capacity(w * h);
            for row in y..y + h {
                let start = row * stride + x * 4;
                pixels.extend(
                    self.bgra[start..start + w * 4]
                        .chunks_exact(4)
                        .map(|p| Color32::from_rgba_premultiplied(p[2], p[1], p[0], p[3])),
                );
            }
            ColorImage::new([w, h], pixels)
        };
        match texture {
            Some(texture) if !std::mem::take(&mut self.resized) => {
                texture.set_partial([x, y], image(x, y, w, h), TextureOptions::NEAREST);
            }
            _ => {
                *texture = Some(ctx.load_texture(
                    name,
                    image(0, 0, self.size[0], self.size[1]),
                    TextureOptions::NEAREST,
                ));
            }
        }
    }
}

fn clamp_rect(rect: &cef::Rect, size: [usize; 2]) -> [usize; 4] {
    let x = usize::try_from(rect.x).unwrap_or(0).min(size[0]);
    let y = usize::try_from(rect.y).unwrap_or(0).min(size[1]);
    let w = usize::try_from(rect.width).unwrap_or(0).min(size[0] - x);
    let h = usize::try_from(rect.height).unwrap_or(0).min(size[1] - y);
    [x, y, w, h]
}

/// What the CEF handlers share with the [`CefView`].
struct Shared {
    ctx: Context,
    tx: UiInboxSender<WebViewEvent>,
    /// The size of the view in points, and points per pixel.
    size: Mutex<(Vec2, f32)>,
    view: Mutex<Layer>,
    popup: Mutex<Layer>,
    /// Where the popup (e.g. of a `<select>`) is, in view points, while it is open.
    popup_rect: Mutex<Option<Rect>>,
    cursor: Mutex<CursorIcon>,
    tooltip: Mutex<Option<String>>,
    /// A text field has focus.
    editable: AtomicBool,
    /// Where the text cursor is, in view points, for the IME's window.
    text_cursor: Mutex<Option<Rect>>,
}

#[derive(Clone)]
struct SharedRef(Arc<Shared>);

wrap_render_handler! {
    struct EguiRenderHandler {
        shared: SharedRef,
    }

    impl RenderHandler {
        fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut cef::Rect>) {
            if let Some(rect) = rect {
                let (size, _) = *self.shared.0.size.lock().expect("poisoned");
                rect.x = 0;
                rect.y = 0;
                rect.width = (size.x.round() as i32).max(1);
                rect.height = (size.y.round() as i32).max(1);
            }
        }

        fn screen_info(
            &self,
            _browser: Option<&mut Browser>,
            screen_info: Option<&mut cef::ScreenInfo>,
        ) -> ::std::os::raw::c_int {
            let Some(screen_info) = screen_info else {
                return 0;
            };
            screen_info.device_scale_factor = self.shared.0.size.lock().expect("poisoned").1;
            1
        }

        fn on_popup_show(&self, _browser: Option<&mut Browser>, show: ::std::os::raw::c_int) {
            if show == 0 {
                *self.shared.0.popup_rect.lock().expect("poisoned") = None;
                *self.shared.0.popup.lock().expect("poisoned") = Layer::default();
                self.shared.0.ctx.request_repaint();
            }
        }

        fn on_popup_size(&self, _browser: Option<&mut Browser>, rect: Option<&cef::Rect>) {
            let Some(rect) = rect else {
                return;
            };
            // Keep the popup inside the view, like cefclient does.
            let (size, _) = *self.shared.0.size.lock().expect("poisoned");
            let popup_size = egui::vec2(rect.width as f32, rect.height as f32);
            let min = pos2(rect.x as f32, rect.y as f32)
                .min((size - popup_size).to_pos2())
                .max(Pos2::ZERO);
            *self.shared.0.popup_rect.lock().expect("poisoned") =
                Some(Rect::from_min_size(min, popup_size));
        }

        fn on_paint(
            &self,
            _browser: Option<&mut Browser>,
            type_: cef::PaintElementType,
            dirty_rects: Option<&[cef::Rect]>,
            buffer: *const u8,
            width: ::std::os::raw::c_int,
            height: ::std::os::raw::c_int,
        ) {
            let (Ok(width), Ok(height)) = (usize::try_from(width), usize::try_from(height)) else {
                return;
            };
            if buffer.is_null() || width == 0 || height == 0 {
                return;
            }
            // SAFETY: CEF hands out a BGRA buffer of `width * height` pixels.
            #[allow(unsafe_code)]
            let buffer = unsafe { std::slice::from_raw_parts(buffer, width * height * 4) };
            let layer = if *type_.as_ref() == sys::cef_paint_element_type_t::PET_POPUP {
                &self.shared.0.popup
            } else {
                &self.shared.0.view
            };
            layer
                .lock()
                .expect("poisoned")
                .paint(buffer, [width, height], dirty_rects.unwrap_or_default());
            self.shared.0.ctx.request_repaint();
        }

        fn on_virtual_keyboard_requested(
            &self,
            _browser: Option<&mut Browser>,
            input_mode: cef::TextInputMode,
        ) {
            let editable = *input_mode.as_ref() != sys::cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_NONE;
            self.shared.0.editable.store(editable, Ordering::Relaxed);
            self.shared.0.ctx.request_repaint();
        }

        fn on_ime_composition_range_changed(
            &self,
            _browser: Option<&mut Browser>,
            _selected_range: Option<&cef::Range>,
            character_bounds: Option<&[cef::Rect]>,
        ) {
            if let Some(last) = character_bounds.and_then(|bounds| bounds.last()) {
                *self.shared.0.text_cursor.lock().expect("poisoned") = Some(Rect::from_min_size(
                    pos2((last.x + last.width) as f32, last.y as f32),
                    egui::vec2(1.0, last.height as f32),
                ));
            }
        }
    }
}

wrap_display_handler! {
    struct EguiDisplayHandler {
        shared: SharedRef,
    }

    impl DisplayHandler {
        fn on_cursor_change(
            &self,
            _browser: Option<&mut Browser>,
            _cursor: *mut u8,
            type_: cef::CursorType,
            _custom_cursor_info: Option<&cef::CursorInfo>,
        ) -> ::std::os::raw::c_int {
            *self.shared.0.cursor.lock().expect("poisoned") = cursor_icon(*type_.as_ref());
            self.shared.0.ctx.request_repaint();
            1
        }

        fn on_tooltip(
            &self,
            _browser: Option<&mut Browser>,
            text: Option<&mut CefString>,
        ) -> ::std::os::raw::c_int {
            let text = text.map(|t| t.to_string()).filter(|t| !t.is_empty());
            *self.shared.0.tooltip.lock().expect("poisoned") = text;
            1
        }

        fn on_console_message(
            &self,
            _browser: Option<&mut Browser>,
            _level: cef::LogSeverity,
            message: Option<&CefString>,
            _source: Option<&CefString>,
            _line: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int {
            let message = message.map(ToString::to_string).unwrap_or_default();
            match message.strip_prefix(IPC_PREFIX) {
                Some(text) => {
                    self.shared.0.tx.send(WebViewEvent::Ipc(text.to_owned())).ok();
                    1
                }
                None => 0,
            }
        }
    }
}

wrap_load_handler! {
    struct EguiLoadHandler {
        shared: SharedRef,
    }

    impl LoadHandler {
        fn on_load_start(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut cef::Frame>,
            _transition_type: cef::TransitionType,
        ) {
            if let Some(url) = main_frame_url(frame) {
                self.shared.0.tx.send(WebViewEvent::Loading(url)).ok();
            }
        }

        fn on_load_end(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut cef::Frame>,
            _http_status_code: ::std::os::raw::c_int,
        ) {
            if let Some(url) = main_frame_url(frame) {
                self.shared.0.tx.send(WebViewEvent::Loaded(url)).ok();
            }
        }
    }
}

fn main_frame_url(frame: Option<&mut cef::Frame>) -> Option<String> {
    let frame = frame.filter(|frame| frame.is_main() == 1)?;
    Some(CefString::from(&frame.url()).to_string())
}

wrap_life_span_handler! {
    struct EguiLifeSpanHandler {}

    impl LifeSpanHandler {
        /// There is nowhere to show a new window, so links that open one load in this view.
        fn on_before_popup(
            &self,
            browser: Option<&mut Browser>,
            _frame: Option<&mut cef::Frame>,
            _popup_id: ::std::os::raw::c_int,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: cef::WindowOpenDisposition,
            _user_gesture: ::std::os::raw::c_int,
            _popup_features: Option<&cef::PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<cef::DictionaryValue>>,
            _no_javascript_access: Option<&mut ::std::os::raw::c_int>,
        ) -> ::std::os::raw::c_int {
            if let (Some(frame), Some(url)) =
                (browser.and_then(|b| b.main_frame()), target_url)
            {
                frame.load_url(Some(url));
            }
            1
        }
    }
}

wrap_context_menu_handler! {
    struct EguiContextMenuHandler {}

    impl ContextMenuHandler {
        /// A windowless browser can't show CEF's native context menu.
        fn on_before_context_menu(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut cef::Frame>,
            _params: Option<&mut cef::ContextMenuParams>,
            model: Option<&mut cef::MenuModel>,
        ) {
            if let Some(model) = model {
                model.clear();
            }
        }
    }
}

wrap_client! {
    struct EguiClient {
        render: RenderHandler,
        display: DisplayHandler,
        load: LoadHandler,
        life_span: LifeSpanHandler,
        context_menu: ContextMenuHandler,
    }

    impl Client {
        fn render_handler(&self) -> Option<RenderHandler> {
            Some(self.render.clone())
        }

        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(self.display.clone())
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            Some(self.load.clone())
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(self.life_span.clone())
        }

        fn context_menu_handler(&self) -> Option<ContextMenuHandler> {
            Some(self.context_menu.clone())
        }
    }
}

fn cursor_icon(cursor: sys::cef_cursor_type_t) -> CursorIcon {
    use sys::cef_cursor_type_t as C;
    match cursor {
        C::CT_CROSS => CursorIcon::Crosshair,
        C::CT_HAND => CursorIcon::PointingHand,
        C::CT_IBEAM => CursorIcon::Text,
        C::CT_VERTICALTEXT => CursorIcon::VerticalText,
        C::CT_WAIT => CursorIcon::Wait,
        C::CT_PROGRESS => CursorIcon::Progress,
        C::CT_HELP => CursorIcon::Help,
        C::CT_EASTRESIZE => CursorIcon::ResizeEast,
        C::CT_NORTHRESIZE => CursorIcon::ResizeNorth,
        C::CT_NORTHEASTRESIZE => CursorIcon::ResizeNorthEast,
        C::CT_NORTHWESTRESIZE => CursorIcon::ResizeNorthWest,
        C::CT_SOUTHRESIZE => CursorIcon::ResizeSouth,
        C::CT_SOUTHEASTRESIZE => CursorIcon::ResizeSouthEast,
        C::CT_SOUTHWESTRESIZE => CursorIcon::ResizeSouthWest,
        C::CT_WESTRESIZE => CursorIcon::ResizeWest,
        C::CT_NORTHSOUTHRESIZE | C::CT_ROWRESIZE => CursorIcon::ResizeVertical,
        C::CT_EASTWESTRESIZE | C::CT_COLUMNRESIZE => CursorIcon::ResizeHorizontal,
        C::CT_NORTHEASTSOUTHWESTRESIZE => CursorIcon::ResizeNeSw,
        C::CT_NORTHWESTSOUTHEASTRESIZE => CursorIcon::ResizeNwSe,
        C::CT_MOVE | C::CT_MIDDLEPANNING => CursorIcon::Move,
        C::CT_CELL => CursorIcon::Cell,
        C::CT_CONTEXTMENU => CursorIcon::ContextMenu,
        C::CT_ALIAS | C::CT_DND_LINK => CursorIcon::Alias,
        C::CT_COPY | C::CT_DND_COPY => CursorIcon::Copy,
        C::CT_NODROP | C::CT_DND_NONE => CursorIcon::NoDrop,
        C::CT_NOTALLOWED => CursorIcon::NotAllowed,
        C::CT_ZOOMIN => CursorIcon::ZoomIn,
        C::CT_ZOOMOUT => CursorIcon::ZoomOut,
        C::CT_GRAB => CursorIcon::Grab,
        C::CT_GRABBING | C::CT_DND_MOVE => CursorIcon::Grabbing,
        C::CT_NONE => CursorIcon::None,
        _ => CursorIcon::Default,
    }
}

/// What the plugin needs to hide a webview that isn't shown.
struct Handle {
    browser: Browser,
    last_shown: Cell<u64>,
    hidden: Cell<bool>,
}

/// A webview rendered by CEF.
pub(crate) struct CefView {
    handle: Rc<Handle>,
    shared: Arc<Shared>,
    texture: Option<TextureHandle>,
    popup_texture: Option<TextureHandle>,
    /// Mouse buttons pressed on the page, which keep getting events outside of it.
    buttons_down: Vec<PointerButton>,
    pointer_inside: bool,
    /// The time, position, button and count of the last click, to count double clicks.
    last_click: Option<(f64, Pos2, PointerButton, i32)>,
}

impl std::fmt::Debug for CefView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CefView").finish_non_exhaustive()
    }
}

impl CefView {
    pub fn new(
        ctx: &Context,
        source: &WebViewSource,
        tx: UiInboxSender<WebViewEvent>,
    ) -> Result<Self, WebViewError> {
        initialize(ctx)?;

        let shared = Arc::new(Shared {
            ctx: ctx.clone(),
            tx,
            size: Mutex::new((egui::vec2(1.0, 1.0), ctx.pixels_per_point())),
            view: Mutex::default(),
            popup: Mutex::default(),
            popup_rect: Mutex::new(None),
            cursor: Mutex::new(CursorIcon::Default),
            tooltip: Mutex::new(None),
            editable: AtomicBool::new(false),
            text_cursor: Mutex::new(None),
        });
        let shared_ref = SharedRef(shared.clone());
        let mut client = EguiClient::new(
            EguiRenderHandler::new(shared_ref.clone()),
            EguiDisplayHandler::new(shared_ref.clone()),
            EguiLoadHandler::new(shared_ref),
            EguiLifeSpanHandler::new(),
            EguiContextMenuHandler::new(),
        );
        let window_info = WindowInfo {
            windowless_rendering_enabled: 1,
            ..Default::default()
        };
        let settings = BrowserSettings {
            windowless_frame_rate: 60,
            ..Default::default()
        };
        let browser = cef::browser_host_create_browser_sync(
            Some(&window_info),
            Some(&mut client),
            Some(&source_url(source).as_str().into()),
            Some(&settings),
            None,
            None,
        )
        .ok_or_else(|| WebViewError::Cef("CEF failed to create a browser".to_owned()))?;

        let handle = Rc::new(Handle {
            browser,
            last_shown: Cell::new(0),
            hidden: Cell::new(false),
        });
        VIEWS.with_borrow_mut(|views| views.push(Rc::downgrade(&handle)));

        Ok(Self {
            handle,
            shared,
            texture: None,
            popup_texture: None,
            buttons_down: Vec::new(),
            pointer_inside: false,
            last_click: None,
        })
    }

    fn browser(&self) -> &Browser {
        &self.handle.browser
    }

    fn host(&self) -> Option<cef::BrowserHost> {
        self.browser().host()
    }

    pub fn load(&self, source: &WebViewSource) {
        if let Some(frame) = self.browser().main_frame() {
            frame.load_url(Some(&source_url(source).as_str().into()));
        }
    }

    pub fn evaluate_script(&self, script: &str) {
        if let Some(frame) = self.browser().main_frame() {
            frame.execute_java_script(Some(&script.into()), None, 0);
        }
    }

    pub fn back(&self) {
        self.browser().go_back();
    }

    pub fn forward(&self) {
        self.browser().go_forward();
    }

    pub fn reload(&self) {
        self.browser().reload();
    }

    /// Show the page in `response.rect`, and give it the input that egui routes there.
    pub fn ui(&mut self, ui: &Ui, response: &Response) {
        let ctx = ui.ctx().clone();
        let Some(host) = self.host() else {
            return;
        };

        self.handle.last_shown.set(ctx.cumulative_pass_nr());
        if self.handle.hidden.replace(false) {
            host.was_hidden(0);
        }

        let rect = response.rect;
        let ppp = ctx.pixels_per_point();
        let (resized, rescaled) = {
            let mut size = self.shared.size.lock().expect("poisoned");
            let changed = (size.0 != rect.size(), size.1 != ppp);
            *size = (rect.size(), ppp);
            changed
        };
        if rescaled {
            host.notify_screen_info_changed();
        }
        if resized || rescaled {
            host.was_resized();
        }

        self.handle_focus(ui, response, &host);
        self.handle_input(ui, response, &host);

        self.shared
            .view
            .lock()
            .expect("poisoned")
            .upload(&ctx, "egui_webview_cef", &mut self.texture);
        let popup_rect = *self.shared.popup_rect.lock().expect("poisoned");
        if popup_rect.is_some() {
            self.shared.popup.lock().expect("poisoned").upload(
                &ctx,
                "egui_webview_cef_popup",
                &mut self.popup_texture,
            );
        }

        let painter = ui.painter_at(rect);
        let uv = Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0));
        if let Some(texture) = &self.texture {
            // At the texture's own size, so a frame from before a resize isn't stretched.
            let size = texture.size_vec2() / ppp;
            painter.image(texture.id(), Rect::from_min_size(rect.min, size), uv, Color32::WHITE);
        }
        if let (Some(popup), Some(texture)) = (popup_rect, &self.popup_texture) {
            painter.image(texture.id(), popup.translate(rect.min.to_vec2()), uv, Color32::WHITE);
        }

        if response.contains_pointer() {
            ctx.set_cursor_icon(*self.shared.cursor.lock().expect("poisoned"));
            if let Some(tooltip) = self.shared.tooltip.lock().expect("poisoned").clone() {
                response.clone().on_hover_text(tooltip);
            }
        }

        if response.has_focus() && self.shared.editable.load(Ordering::Relaxed) {
            let cursor = self
                .shared
                .text_cursor
                .lock()
                .expect("poisoned")
                .map_or(rect, |cursor| cursor.translate(rect.min.to_vec2()));
            ctx.output_mut(|o| {
                o.ime = Some(egui::output::IMEOutput {
                    rect,
                    cursor_rect: cursor,
                    purpose: egui::IMEPurpose::Normal,
                    should_interrupt_composition: false,
                });
            });
        }
    }

    fn handle_focus(&self, ui: &Ui, response: &Response, host: &cef::BrowserHost) {
        if response.gained_focus() {
            host.set_focus(1);
            self.shared.tx.send(WebViewEvent::Focus).ok();
        }
        if response.lost_focus() {
            host.set_focus(0);
            self.shared.tx.send(WebViewEvent::Blur).ok();
        }
        if response.clicked_elsewhere() && response.has_focus() {
            response.surrender_focus();
        }
        if response.has_focus() {
            // The page handles tab, arrows and escape itself.
            ui.memory_mut(|mem| {
                mem.set_focus_lock_filter(
                    response.id,
                    EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                );
            });
        }
    }

    #[expect(clippy::too_many_lines)] // One arm per kind of event.
    fn handle_input(&mut self, ui: &Ui, response: &Response, host: &cef::BrowserHost) {
        let rect = response.rect;
        // `contains_pointer` is false while a layer above the page has the pointer.
        let hovered = response.contains_pointer();
        let focused = response.has_focus();
        let (events, time, hover_pos, modifiers) = ui.input(|i| {
            (
                i.events.clone(),
                i.time,
                i.pointer.hover_pos(),
                i.modifiers,
            )
        });
        let has_ime_commit = events
            .iter()
            .any(|e| matches!(e, Event::Ime(egui::ImeEvent::Commit(_))));

        let mouse = |pos: Pos2, modifiers: Modifiers, buttons: &[PointerButton]| {
            let pos = pos - rect.min;
            MouseEvent {
                x: pos.x.round() as i32,
                y: pos.y.round() as i32,
                modifiers: event_flags(modifiers) | button_flags(buttons),
            }
        };

        for event in events {
            match event {
                Event::PointerMoved(pos) => {
                    if hovered || !self.buttons_down.is_empty() {
                        let event = mouse(pos, modifiers, &self.buttons_down);
                        host.send_mouse_move_event(Some(&event), 0);
                        self.pointer_inside = true;
                    } else if std::mem::take(&mut self.pointer_inside) {
                        let event = mouse(pos, modifiers, &[]);
                        host.send_mouse_move_event(Some(&event), 1);
                    }
                }
                Event::PointerGone if std::mem::take(&mut self.pointer_inside) => {
                    let event = mouse(hover_pos.unwrap_or(rect.min), modifiers, &[]);
                    host.send_mouse_move_event(Some(&event), 1);
                }
                Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers,
                } => {
                    let Some(cef_button) = mouse_button(button) else {
                        continue;
                    };
                    if pressed && hovered {
                        response.request_focus();
                        let count = self.click_count(time, pos, button);
                        self.buttons_down.push(button);
                        let event = mouse(pos, modifiers, &self.buttons_down);
                        host.send_mouse_click_event(Some(&event), cef_button, 0, count);
                    } else if !pressed && self.buttons_down.contains(&button) {
                        self.buttons_down.retain(|b| *b != button);
                        let count = self.last_click.map_or(1, |(.., count)| count);
                        let event = mouse(pos, modifiers, &self.buttons_down);
                        host.send_mouse_click_event(Some(&event), cef_button, 1, count);
                    }
                }
                Event::MouseWheel {
                    unit,
                    delta,
                    modifiers,
                    ..
                } if hovered => {
                    let delta = match unit {
                        egui::MouseWheelUnit::Point => delta,
                        egui::MouseWheelUnit::Line => delta * POINTS_PER_LINE,
                        egui::MouseWheelUnit::Page => delta * rect.height(),
                    };
                    let event = mouse(hover_pos.unwrap_or(rect.min), modifiers, &[]);
                    host.send_mouse_wheel_event(
                        Some(&event),
                        delta.x.round() as i32,
                        delta.y.round() as i32,
                    );
                }
                Event::Key {
                    key,
                    physical_key,
                    pressed,
                    repeat,
                    modifiers,
                } if focused => {
                    send_key(host, key, physical_key, pressed, repeat, modifiers);
                }
                Event::Text(text) if focused && !has_ime_commit => {
                    for c in text.encode_utf16() {
                        send_char(host, c, modifiers);
                    }
                }
                Event::Ime(egui::ImeEvent::Preedit { text, .. }) if focused => {
                    let end = u32::try_from(text.encode_utf16().count()).unwrap_or(0);
                    let selection = cef::Range { from: end, to: end };
                    host.ime_set_composition(
                        Some(&text.as_str().into()),
                        None,
                        None,
                        Some(&selection),
                    );
                }
                Event::Ime(egui::ImeEvent::Commit(text)) if focused => {
                    host.ime_commit_text(Some(&text.as_str().into()), None, 0);
                }
                Event::Copy if focused => self.with_focused_frame(ImplFrame::copy),
                Event::Cut if focused => self.with_focused_frame(ImplFrame::cut),
                // CEF reads the clipboard itself.
                Event::Paste(_) if focused => self.with_focused_frame(ImplFrame::paste),
                _ => {}
            }
        }
    }

    fn with_focused_frame(&self, f: impl FnOnce(&cef::Frame)) {
        if let Some(frame) = self.browser().focused_frame() {
            f(&frame);
        }
    }

    /// How many clicks in a row this one is, for double and triple clicks.
    fn click_count(&mut self, time: f64, pos: Pos2, button: PointerButton) -> i32 {
        const MAX_DELAY: f64 = 0.5;
        const MAX_DISTANCE: f32 = 4.0;
        let count = match self.last_click {
            Some((last_time, last_pos, last_button, count))
                if last_button == button
                    && time - last_time < MAX_DELAY
                    && last_pos.distance(pos) < MAX_DISTANCE =>
            {
                count + 1
            }
            _ => 1,
        };
        self.last_click = Some((time, pos, button, count));
        count
    }
}

impl Drop for CefView {
    fn drop(&mut self) {
        if let Some(host) = self.host() {
            host.close_browser(1);
        }
    }
}

fn send_key(
    host: &cef::BrowserHost,
    key: egui::Key,
    physical_key: Option<egui::Key>,
    pressed: bool,
    repeat: bool,
    modifiers: Modifiers,
) {
    let Some(codes) = keys::key_codes(key, physical_key) else {
        return;
    };
    let mut flags = event_flags(modifiers);
    if repeat {
        flags |= sys::cef_event_flags_t::EVENTFLAG_IS_REPEAT.0;
    }
    let type_ = if pressed {
        sys::cef_key_event_type_t::KEYEVENT_RAWKEYDOWN
    } else {
        sys::cef_key_event_type_t::KEYEVENT_KEYUP
    };
    host.send_key_event(Some(&KeyEvent {
        size: std::mem::size_of::<KeyEvent>(),
        type_: type_.into(),
        modifiers: flags,
        windows_key_code: codes.windows,
        native_key_code: codes.native,
        ..Default::default()
    }));
    // egui sends no text for enter, but pages expect a character for it.
    if pressed && key == egui::Key::Enter {
        send_char(host, u16::from(b'\r'), modifiers);
    }
}

fn send_char(host: &cef::BrowserHost, c: u16, modifiers: Modifiers) {
    host.send_key_event(Some(&KeyEvent {
        size: std::mem::size_of::<KeyEvent>(),
        type_: sys::cef_key_event_type_t::KEYEVENT_CHAR.into(),
        modifiers: event_flags(modifiers),
        windows_key_code: i32::from(c),
        character: c,
        unmodified_character: c,
        ..Default::default()
    }));
}

fn event_flags(modifiers: Modifiers) -> u32 {
    use sys::cef_event_flags_t as F;
    let mut flags = 0;
    for (on, flag) in [
        (modifiers.shift, F::EVENTFLAG_SHIFT_DOWN),
        (modifiers.ctrl, F::EVENTFLAG_CONTROL_DOWN),
        (modifiers.alt, F::EVENTFLAG_ALT_DOWN),
        (modifiers.mac_cmd, F::EVENTFLAG_COMMAND_DOWN),
    ] {
        if on {
            flags |= flag.0;
        }
    }
    flags
}

fn button_flags(buttons: &[PointerButton]) -> u32 {
    use sys::cef_event_flags_t as F;
    buttons
        .iter()
        .map(|button| match button {
            PointerButton::Primary => F::EVENTFLAG_LEFT_MOUSE_BUTTON.0,
            PointerButton::Secondary => F::EVENTFLAG_RIGHT_MOUSE_BUTTON.0,
            PointerButton::Middle => F::EVENTFLAG_MIDDLE_MOUSE_BUTTON.0,
            _ => 0,
        })
        .fold(0, |a, b| a | b)
}

fn mouse_button(button: PointerButton) -> Option<cef::MouseButtonType> {
    use sys::cef_mouse_button_type_t as B;
    Some(
        match button {
            PointerButton::Primary => B::MBT_LEFT,
            PointerButton::Secondary => B::MBT_RIGHT,
            PointerButton::Middle => B::MBT_MIDDLE,
            _ => return None,
        }
        .into(),
    )
}

/// CEF only loads URLs, so HTML becomes a `data:` URL.
fn source_url(source: &WebViewSource) -> String {
    match source {
        WebViewSource::Url(url) => url.clone(),
        WebViewSource::Html(html) => {
            let mut url = String::from("data:text/html;charset=utf-8,");
            for byte in html.bytes() {
                if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
                    url.push(char::from(byte));
                } else {
                    write!(url, "%{byte:02X}").ok();
                }
            }
            url
        }
    }
}
