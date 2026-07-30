use egui::{Context, Id};
use egui_wgpu::RenderState;

/// Wrapper so the render state can live in [`egui::Context`]'s data store.
#[derive(Clone)]
struct Installed(RenderState);

fn id() -> Id {
    Id::new("regui_wgpu_render_state")
}

/// Give `regui` access to wgpu.
///
/// Call this once at startup. Effects such as [`crate::BackdropBlur`] need the device and
/// the target format, and there is no way to reach them from inside a widget.
///
/// ```no_run
/// # use eframe::CreationContext;
/// fn setup(cc: &CreationContext<'_>) {
///     if let Some(render_state) = cc.wgpu_render_state.clone() {
///         regui::install_wgpu(&cc.egui_ctx, render_state);
///     }
/// }
/// ```
///
/// Anything that needs it says so, and does nothing but log a warning if you forget.
pub fn install_wgpu(ctx: &Context, render_state: RenderState) {
    ctx.data_mut(|data| data.insert_temp(id(), Installed(render_state)));
}

/// The render state [`install_wgpu`] was given, if it was called.
pub(crate) fn render_state(ctx: &Context) -> Option<RenderState> {
    ctx.data_mut(|data| data.get_temp::<Installed>(id()))
        .map(|installed| installed.0)
}

/// Complain once per context that [`install_wgpu`] was never called.
pub(crate) fn warn_not_installed(ctx: &Context, what: &str) {
    let warned = Id::new("regui_warned_no_wgpu");
    if ctx.data_mut(|data| data.get_temp::<bool>(warned).unwrap_or(false)) {
        return;
    }
    ctx.data_mut(|data| data.insert_temp(warned, true));
    log::warn!(
        "regui: {what} needs wgpu, but `regui::install_wgpu` was never called, so it will \
         not be drawn. Call it once at startup with `cc.wgpu_render_state`."
    );
}
