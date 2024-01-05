use egui::{Window};
use egui_webview::{init_webview, webview_end_frame};


pub fn main() -> eframe::Result<()> {
  let mut view = None;

  eframe::run_simple_native("Dnd Example App", Default::default(), move |ctx, _frame| {
    if view.is_none() {
      init_webview(ctx, _frame);

      view = Some(egui_webview::EguiWebView::new(ctx, "webview", |b| {
        b.with_url("https://www.google.com").unwrap()
      }));
    }

    let view = view.as_mut().unwrap();

    Window::new("Hello").show(ctx, |ui| {
      view.ui(ui, ui.available_size());
    });

    Window::new("Hello 2").show(ctx, |ui| {
      ui.label("Hello World!");
    });

    webview_end_frame(ctx);
  })
}
