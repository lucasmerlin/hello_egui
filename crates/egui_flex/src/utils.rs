use egui::emath::TSTransform;
use egui::layers::ShapeIdx;
use egui::Ui;

pub fn with_visual_transform<R>(
    ui: &mut Ui,
    transform: TSTransform,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let start_idx = ui.ctx().graphics(|gx| {
        gx.get(ui.layer_id())
            .map_or(ShapeIdx(0), egui::layers::PaintList::next_idx)
    });

    let r = add_contents(ui);

    ui.ctx().graphics_mut(|g| {
        let list = g.entry(ui.layer_id());
        let end_idx = list.next_idx();
        list.transform_range(start_idx, end_idx, transform);
    });

    r
}
