//! A hosted viewport's pass leaves this frame's texture uploads in the `Context`, for the
//! viewport hosting it to hand to its backend. These check that running a child does not
//! disturb the parent's own textures, which an earlier version did by draining the uploads
//! and replaying them: a freed texture cannot be re-queued, since `TextureManager::free`
//! decrements a retain count that has already reached zero.

use egui::{Color32, ColorImage, TextureHandle, TextureOptions, Ui, load::SizedTexture, vec2};
use egui_kittest::Harness;
use regui::Regui;

fn image(color: Color32) -> ColorImage {
    ColorImage::filled([8, 8], color)
}

/// Loading and dropping a texture during a pass that also runs a child must not upset the
/// texture manager.
///
/// A dropped `TextureHandle` frees its texture, which puts it in the pass' free list, and a
/// texture allocated and dropped in the same pass ends up in both lists. Replaying either
/// one trips a debug assert in `TextureManager`.
#[test]
fn dropping_a_texture_while_a_child_runs_is_fine() {
    let mut keep: Option<TextureHandle> = None;
    let mut pass = 0_u32;

    let mut harness = Harness::new_ui(move |ui| {
        pass += 1;

        // Load one texture and hold it, and another that is dropped straight away, so that
        // this pass has both an allocation and a free in it.
        if pass == 2 {
            keep = Some(
                ui.ctx()
                    .load_texture("kept", image(Color32::RED), TextureOptions::LINEAR),
            );
        }
        if pass == 3 {
            let dropped =
                ui.ctx()
                    .load_texture("dropped", image(Color32::GREEN), TextureOptions::LINEAR);
            drop(dropped);
        }
        if pass == 4 {
            // Drop the one we kept, which frees it during this pass.
            assert!(keep.is_some(), "should still be holding the kept texture");
            keep = None;
        }
        // `keep` is only here to control when the drop happens, but read it so the compiler
        // can see that.
        assert_eq!(keep.is_some(), (2..4).contains(&pass));

        Regui::new("child").size(vec2(80.0, 40.0)).show(ui, |ui| {
            ui.label("child");
        });
    });

    // Any of these passes would trip a debug assert in `TextureManager` if the delta were
    // handed back wrongly.
    for _ in 0..6 {
        harness.run();
    }
}

/// A texture the parent draws must still be there after a child has run.
#[test]
fn the_parent_keeps_its_textures_when_a_child_runs() {
    let mut texture: Option<TextureHandle> = None;

    let mut harness = Harness::new_ui(move |ui| {
        let texture = texture.get_or_insert_with(|| {
            ui.ctx()
                .load_texture("parent", image(Color32::RED), TextureOptions::NEAREST)
        });

        ui.image(SizedTexture::new(texture.id(), vec2(32.0, 32.0)));

        Regui::new("child").size(vec2(80.0, 40.0)).show(ui, |ui| {
            ui.label("child");
        });

        // The handle is still alive, so the texture must still be allocated.
        assert!(
            ui.ctx().tex_manager().read().meta(texture.id()).is_some(),
            "the parent's texture was freed while a child was running"
        );
    });

    for _ in 0..4 {
        harness.run();
    }
}

/// The same, through the off-screen renderer, which applies the pending uploads itself
/// before rendering the child, without taking them.
#[cfg(feature = "wgpu")]
#[test]
fn the_offscreen_path_forwards_textures_too() {
    use egui_kittest::wgpu::{WgpuTestRenderer, create_render_state, default_wgpu_setup};

    let render_state = create_render_state(
        default_wgpu_setup(),
        egui_wgpu::RendererOptions::PREDICTABLE,
    );
    let installed = render_state.clone();

    let mut texture: Option<TextureHandle> = None;
    let mut pass = 0_u32;

    let mut harness = Harness::builder()
        .with_size([200.0, 160.0])
        .renderer(WgpuTestRenderer::from_render_state(render_state))
        .build_ui(move |ui: &mut Ui| {
            regui::install_wgpu(ui.ctx(), installed.clone());
            pass += 1;

            let texture = texture.get_or_insert_with(|| {
                ui.ctx()
                    .load_texture("parent", image(Color32::RED), TextureOptions::NEAREST)
            });
            ui.image(SizedTexture::new(texture.id(), vec2(32.0, 32.0)));

            if pass == 3 {
                let dropped = ui.ctx().load_texture(
                    "dropped",
                    image(Color32::GREEN),
                    TextureOptions::NEAREST,
                );
                drop(dropped);
            }

            Regui::new("child")
                .size(vec2(80.0, 40.0))
                .offscreen(true)
                .show(ui, |ui| {
                    ui.label("child");
                });
        });

    for _ in 0..4 {
        harness.run();
    }
    // Rendering is what would fail if a texture the parent still draws had been freed.
    harness.render().expect("failed to render");
}
