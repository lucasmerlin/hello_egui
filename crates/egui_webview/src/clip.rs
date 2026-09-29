//! Keeps the native webview out from under the egui layers painted above it.
//!
//! The webview is a native view on top of egui's surface, so nothing egui paints
//! can cover it. There are two ways around that:
//!
//! - Holes: each layer above the webview's layer cuts a hole into the native
//!   view, through which egui's own rendering of that layer shows.
//! - Paint planes: egui paints the layers above the webview a second time, into a
//!   transparent surface over it (see [`egui::Context::add_paint_plane`]).
//!
//! Either way, clicks in the rects of those layers go through to egui.

use egui::{Context, Id, LayerId, Rect};
use std::collections::HashMap;

/// Where a webview goes this frame, in window logical points.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Placement {
    /// The full rect of the page.
    pub rect: Rect,
    /// The part of `rect` inside the clip rect of its `Ui`.
    pub visible: Rect,
    /// Rects inside `visible` that egui paints over.
    pub holes: Vec<Rect>,
    /// A modal is open above the webview.
    pub under_modal: bool,
    /// Where the webview's layer is in egui's paint order. Native views have to
    /// stack the same way, or a webview in a window behind covers one in front.
    pub stack: usize,
}

/// What [`crate::EguiWebView::ui`] saw of a webview this frame, in egui points.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Shown {
    pub layer: LayerId,
    pub rect: Rect,
    /// The part of `rect` inside the clip rect, rounded to pixels.
    pub visible: Rect,
}

/// Turn what each webview saw this frame into a placement.
///
/// Call it once all areas have been shown, so the rects of popups opened
/// after the webview are already known.
pub(crate) fn placements(ctx: &Context, shown: &HashMap<Id, Shown>) -> HashMap<Id, Placement> {
    let zoom = ctx.zoom_factor();
    ctx.memory(|mem| {
        let order = mem.layer_ids().collect::<Vec<_>>();
        shown
            .iter()
            .map(|(id, shown)| {
                let above = order
                    .iter()
                    .rposition(|layer| *layer == shown.layer)
                    .map_or(order.len(), |i| i + 1);
                let holes = order[above..]
                    .iter()
                    .filter(|layer| mem.areas().is_visible(layer))
                    .filter_map(|layer| mem.area_rect(layer.id))
                    .map(|rect| rect.intersect(shown.visible))
                    .filter(Rect::is_positive)
                    .map(|rect| rect * zoom)
                    .collect();
                let placement = Placement {
                    rect: shown.rect * zoom,
                    visible: shown.visible * zoom,
                    holes,
                    // One frame behind: `top_modal_layer` is only set at the
                    // end of the pass.
                    under_modal: !mem.is_above_modal_layer(shown.layer),
                    stack: above,
                };
                (*id, placement)
            })
            .collect()
    })
}

/// Cut `holes` out of `rect`. The result is a set of rects that do not overlap.
#[cfg_attr(
    not(any(target_os = "macos", target_arch = "wasm32")),
    allow(dead_code)
)]
pub(crate) fn subtract(rect: Rect, holes: &[Rect]) -> Vec<Rect> {
    let mut parts = vec![rect];
    for hole in holes {
        parts = parts
            .into_iter()
            .flat_map(|part| {
                let cut = part.intersect(*hole);
                if !cut.is_positive() {
                    return vec![part];
                }
                // The strips above and below the cut span the full width, the
                // strips left and right of it only its height.
                [
                    Rect::from_x_y_ranges(part.x_range(), part.min.y..=cut.min.y),
                    Rect::from_x_y_ranges(part.x_range(), cut.max.y..=part.max.y),
                    Rect::from_x_y_ranges(part.min.x..=cut.min.x, cut.y_range()),
                    Rect::from_x_y_ranges(cut.max.x..=part.max.x, cut.y_range()),
                ]
                .into_iter()
                .filter(Rect::is_positive)
                .collect()
            })
            .collect();
    }
    parts
}

#[cfg(target_os = "macos")]
pub(crate) use macos::NativeClip;

#[cfg(not(any(target_os = "macos", target_arch = "wasm32")))]
pub(crate) use fallback::NativeClip;

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos {
    use std::cell::RefCell;

    use objc2::rc::Retained;
    use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
    use objc2_app_kit::NSView;
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_core_graphics::CGMutablePath;
    use objc2_quartz_core::{CAShapeLayer, CATransaction};
    use std::ptr::NonNull;
    use wry::raw_window_handle::{
        AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
    };
    use wry::WebViewExtMacOS;

    use super::{subtract, Placement};

    fn cg_rect(rect: egui::Rect) -> CGRect {
        CGRect::new(
            CGPoint::new(f64::from(rect.min.x), f64::from(rect.min.y)),
            CGSize::new(f64::from(rect.width()), f64::from(rect.height())),
        )
    }

    #[derive(Default)]
    struct ClipViewIvars {
        /// In the view's own (flipped) coordinates.
        holes: RefCell<Vec<egui::Rect>>,
    }

    define_class!(
        /// Holds the `WKWebView`, crops it to the visible part of its `Ui` and
        /// cuts the holes into it.
        #[unsafe(super(NSView))]
        #[thread_kind = MainThreadOnly]
        #[ivars = ClipViewIvars]
        struct ClipView;

        impl ClipView {
            /// Same as winit's view, so both use egui's top-left origin.
            #[unsafe(method(isFlipped))]
            fn is_flipped(&self) -> bool {
                true
            }

            /// A click in a hole goes on to the winit view below, and so to egui.
            #[unsafe(method_id(hitTest:))]
            fn hit_test(&self, point: CGPoint) -> Option<Retained<NSView>> {
                let local = self.convertPoint_fromView(point, unsafe { self.superview() }.as_deref());
                let local = egui::pos2(local.x as f32, local.y as f32);
                if self.ivars().holes.borrow().iter().any(|hole| hole.contains(local)) {
                    None
                } else {
                    unsafe { msg_send![super(self), hitTest: point] }
                }
            }
        }
    );

    impl ClipView {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(ClipViewIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    define_class!(
        /// Shows the paint plane over the `WKWebView`. Clicks go through it.
        #[unsafe(super(NSView))]
        #[thread_kind = MainThreadOnly]
        struct PlaneView;

        impl PlaneView {
            #[unsafe(method(isFlipped))]
            fn is_flipped(&self) -> bool {
                true
            }

            #[unsafe(method_id(hitTest:))]
            fn hit_test(&self, _point: CGPoint) -> Option<Retained<NSView>> {
                None
            }
        }
    );

    impl PlaneView {
        fn new(mtm: MainThreadMarker) -> Retained<Self> {
            let this = Self::alloc(mtm).set_ivars(());
            unsafe { msg_send![super(this), init] }
        }
    }

    /// The transparent view egui-wgpu paints the webview's paint plane into.
    struct Plane {
        id: egui::Id,
        view: Retained<PlaneView>,
        surfaces: egui_wgpu::PlaneSurfaces,
    }

    impl Plane {
        fn new(
            mtm: MainThreadMarker,
            id: egui::Id,
            render_state: &egui_wgpu::RenderState,
        ) -> Option<Self> {
            let view = PlaneView::new(mtm);
            let handle = AppKitWindowHandle::new(NonNull::from(&*view).cast());
            let target = egui_wgpu::wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(RawDisplayHandle::AppKit(AppKitDisplayHandle::new())),
                raw_window_handle: RawWindowHandle::AppKit(handle),
            };
            // The view outlives the surface: `Drop` removes the surface first.
            let surface = unsafe { render_state.instance.create_surface_unsafe(target) }
                .inspect_err(|err| eprintln!("egui_webview: no paint plane surface: {err}"))
                .ok()?;
            render_state.plane_surfaces.insert(id, surface);
            Some(Self {
                id,
                view,
                surfaces: render_state.plane_surfaces.clone(),
            })
        }
    }

    impl Drop for Plane {
        fn drop(&mut self) {
            self.surfaces.remove(self.id);
            self.view.removeFromSuperview();
        }
    }

    pub(crate) struct NativeClip {
        view: Retained<ClipView>,
        mask: Retained<CAShapeLayer>,
        /// With paint planes, instead of holes in the mask.
        plane: Option<Plane>,
        /// `None` while hidden, which is how the view starts.
        last: RefCell<Option<Placement>>,
    }

    impl NativeClip {
        /// Move the `WKWebView` from winit's view into a [`ClipView`].
        ///
        /// With a `render_state`, egui paints the layers above the webview into a
        /// plane over it. Without one, they cut holes into it.
        pub fn new(
            webview: &wry::WebView,
            id: egui::Id,
            render_state: Option<&egui_wgpu::RenderState>,
        ) -> Self {
            let mtm = MainThreadMarker::new().expect("webviews live on the main thread");
            let wk = webview.webview();
            let parent = unsafe { wk.superview() }.expect("wry adds child webviews to a view");

            let view = ClipView::new(mtm);
            view.setWantsLayer(true);
            view.setHidden(true);
            parent.addSubview(&view);
            wk.removeFromSuperview();
            view.addSubview(&wk);

            let plane = render_state.and_then(|render_state| Plane::new(mtm, id, render_state));
            if let Some(plane) = &plane {
                view.addSubview(&plane.view);
            }

            Self {
                view,
                mask: CAShapeLayer::new(),
                plane,
                last: RefCell::new(None),
            }
        }

        pub fn has_plane(&self) -> bool {
            self.plane.is_some()
        }

        pub fn update(&self, webview: &wry::WebView, placement: Option<&Placement>) {
            // Holes can't show a modal's backdrop, but a plane can.
            let placement = placement
                .filter(|p| p.visible.is_positive() && (self.plane.is_some() || !p.under_modal));
            if self.last.borrow().as_ref() == placement {
                return;
            }
            self.last.replace(placement.cloned());

            let Some(placement) = placement else {
                self.view.setHidden(true);
                return;
            };

            let origin = placement.visible.min.to_vec2();
            let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, placement.visible.size());
            let holes = if placement.under_modal {
                // The modal takes all input.
                vec![bounds]
            } else {
                placement
                    .holes
                    .iter()
                    .map(|hole| hole.translate(-origin))
                    .collect::<Vec<_>>()
            };

            // A layer that no view owns animates every change by default.
            CATransaction::begin();
            CATransaction::setDisableActions(true);

            self.view.setFrame(cg_rect(placement.visible));
            webview
                .webview()
                .setFrame(cg_rect(placement.rect.translate(-origin)));

            if let Some(plane) = &self.plane {
                plane.view.setFrame(cg_rect(bounds));
            }

            let layer = self.view.layer();
            if let Some(layer) = &layer {
                // Above egui's own surface, which sits at 0. Only drawing follows
                // this; clicks don't need to, since the holes of a webview behind
                // cover the windows in front of it.
                layer.setZPosition(1.0 + placement.stack as f64);
            }
            if holes.is_empty() || self.plane.is_some() {
                if let Some(layer) = layer {
                    unsafe { layer.setMask(None) };
                }
            } else if let Some(layer) = layer {
                let path = CGMutablePath::new();
                for part in subtract(bounds, &holes) {
                    unsafe {
                        CGMutablePath::add_rect(Some(&path), std::ptr::null(), cg_rect(part));
                    };
                }
                self.mask.setFrame(cg_rect(bounds));
                self.mask.setPath(Some(&path));
                unsafe { layer.setMask(Some(&self.mask)) };
            }

            CATransaction::commit();

            *self.view.ivars().holes.borrow_mut() = holes;
            self.view.setHidden(false);
        }
    }

    impl Drop for NativeClip {
        fn drop(&mut self) {
            self.view.removeFromSuperview();
        }
    }
}

#[cfg(not(any(target_os = "macos", target_arch = "wasm32")))]
mod fallback {
    use wry::dpi::{LogicalPosition, LogicalSize};

    use super::Placement;

    /// No holes or planes on this platform yet: hide the webview while anything covers it.
    pub(crate) struct NativeClip;

    impl NativeClip {
        pub fn new(
            _webview: &wry::WebView,
            _id: egui::Id,
            _render_state: Option<&egui_wgpu::RenderState>,
        ) -> Self {
            Self
        }

        #[allow(clippy::unused_self)]
        pub fn has_plane(&self) -> bool {
            false
        }

        #[allow(clippy::unused_self)]
        pub fn update(&self, webview: &wry::WebView, placement: Option<&Placement>) {
            let Some(placement) = placement.filter(|p| p.holes.is_empty() && !p.under_modal) else {
                webview.set_visible(false).ok();
                return;
            };
            webview
                .set_bounds(wry::Rect {
                    position: LogicalPosition::new(
                        f64::from(placement.rect.min.x),
                        f64::from(placement.rect.min.y),
                    )
                    .into(),
                    size: LogicalSize::new(
                        f64::from(placement.rect.width()),
                        f64::from(placement.rect.height()),
                    )
                    .into(),
                })
                .ok();
            webview.set_visible(true).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{pos2, Rect};

    use super::subtract;

    fn area(rects: &[Rect]) -> f32 {
        rects.iter().map(Rect::area).sum()
    }

    #[test]
    fn no_holes_keeps_the_rect() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 50.0));
        assert_eq!(subtract(rect, &[]), vec![rect]);
    }

    #[test]
    fn overlapping_holes_stay_cut() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 100.0));
        let a = Rect::from_min_max(pos2(10.0, 10.0), pos2(50.0, 50.0));
        let b = Rect::from_min_max(pos2(30.0, 30.0), pos2(70.0, 70.0));
        let parts = subtract(rect, &[a, b]);

        // 10_000 - 1600 - 1600 + 400 of overlap.
        assert_eq!(area(&parts), 7200.0);
        for part in &parts {
            assert!(!part.intersects(a) || part.intersect(a).area() == 0.0);
            assert!(!part.intersects(b) || part.intersect(b).area() == 0.0);
        }
    }

    #[test]
    fn hole_outside_changes_nothing() {
        let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(10.0, 10.0));
        let hole = Rect::from_min_max(pos2(20.0, 20.0), pos2(30.0, 30.0));
        assert_eq!(subtract(rect, &[hole]), vec![rect]);
    }
}
