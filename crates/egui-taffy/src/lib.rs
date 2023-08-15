use std::mem;

use egui::util::IdTypeMap;
use egui::{Context, Id, LayerId, Order, Pos2, Ui};
use taffy::prelude::*;
use taffy::Taffy;

type Node = NodeId;

struct TaffyState {
    taffy: Taffy<MeasureFunc<&'static mut Vec<ContentFn<'static>>>>,
    ctx: Context,

    children: Vec<(Id, Node, egui::Layout)>,

    node: Node,
    node_style: Style,

    last_size: egui::Vec2,
}

impl Clone for TaffyState {
    fn clone(&self) -> Self {
        panic!("TaffyState is not cloneable")
    }
}

unsafe impl Send for TaffyState {}
unsafe impl Sync for TaffyState {}

impl TaffyState {
    pub fn new(ctx: Context) -> Self {
        let mut taffy = Taffy::new();

        Self {
            node: taffy.new_with_children(Style::default(), &[]).unwrap(),
            node_style: Style::default(),
            taffy,
            ctx,
            children: Vec::new(),
            last_size: egui::Vec2::ZERO,
        }
    }

    pub fn begin_pass(&mut self, style: Style) {
        if self.node_style != style {
            dbg!("Setting style");
            self.node_style = style.clone();
            self.taffy.set_style(self.node, style).unwrap();
        }
    }
}

type ContentFn<'a> = Box<dyn FnMut(&mut Ui) + Send + 'a>;

pub struct TaffyPass<'a> {
    id: Id,

    ui: &'a mut Ui,

    content_fns: Vec<Box<dyn FnMut(&mut Ui) + Send + 'a>>,

    measure_ctx: Context,
}

impl<'a> TaffyPass<'a> {
    fn with_state<T>(id: Id, ctx: Context, f: impl FnOnce(&mut TaffyState) -> T) -> T {
        ctx.data_mut(|data: &mut IdTypeMap| {
            let data = data.get_temp_mut_or_insert_with(id, || TaffyState::new(ctx.clone()));

            f(data)
        })
    }

    pub fn new(ui: &'a mut Ui, id: Id, style: Style) -> Self {
        Self::with_state(id, ui.ctx().clone(), |state| {
            state.begin_pass(style);
        });

        let measure_ctx = Context::default();
        measure_ctx.run(Default::default(), |_| {});

        Self {
            id,
            ui,
            content_fns: vec![],
            measure_ctx,
        }
    }

    pub fn child(
        mut self,
        id: Id,
        style: Style,
        layout: egui::Layout,
        content: impl FnMut(&mut Ui) + Send + 'a,
    ) -> TaffyPass<'a> {
        Self::with_state(self.id, self.ui.ctx().clone(), |state| {
            let idx = self.content_fns.len();

            self.content_fns.push(Box::new(content));

            if let Some((c_id, c_node, c_layout)) = state.children.get_mut(idx) {
                if *c_id != id
                    || state.taffy.style(c_node.clone()).unwrap() != &style
                    || *c_layout != layout
                {
                    *c_id = id;
                    *c_layout = layout;
                    state.taffy.set_style(*c_node, style).unwrap();
                }
            } else {
                let ctx = self.measure_ctx.clone();

                let node = state
                    .taffy
                    .new_leaf_with_measure(
                        style,
                        MeasureFunc::Boxed(Box::new(
                            move |known_size: Size<Option<f32>>,
                                  avaiable_space: Size<AvailableSpace>,
                                  data|
                                  -> Size<f32> {
                                let f = data.get_mut(idx).unwrap();

                                let available_width = match avaiable_space.width {
                                    AvailableSpace::Definite(num) => num,
                                    AvailableSpace::MinContent => 0.0,
                                    AvailableSpace::MaxContent => f32::MAX,
                                };

                                let available_height = match avaiable_space.height {
                                    AvailableSpace::Definite(num) => num,
                                    AvailableSpace::MinContent => 0.0,
                                    AvailableSpace::MaxContent => f32::MAX,
                                };

                                let rect = egui::Rect::from_min_size(
                                    Default::default(),
                                    egui::Vec2::new(
                                        known_size.width.unwrap_or(available_width),
                                        known_size.height.unwrap_or(available_height),
                                    ),
                                );

                                let mut ui = Ui::new(
                                    ctx.clone(),
                                    LayerId::new(Order::Background, Id::new("measure")),
                                    Id::new("measure"),
                                    rect,
                                    egui::Rect::from_min_size(
                                        Default::default(),
                                        Default::default(),
                                    ),
                                );
                                let response = ui.with_layout(
                                    egui::Layout {
                                        main_dir: layout.main_dir,
                                        ..egui::Layout::default()
                                    },
                                    |ui| {
                                        f(ui);
                                    },
                                );

                                let result_rect = response.response.rect;

                                Size {
                                    width: result_rect.width().ceil(),
                                    height: result_rect.height(),
                                }
                            },
                        )),
                    )
                    .unwrap();

                if idx >= state.children.len() {
                    state.children.push((id, node, layout));
                } else {
                    state.children[idx] = ((id, node, layout));
                }

                state
                    .taffy
                    .set_children(
                        state.node,
                        &state
                            .children
                            .iter()
                            .map(|(id, node, layout)| node.clone())
                            .collect::<Vec<_>>(),
                    )
                    .unwrap();
            }
        });
        self
    }

    pub fn show(mut self) {
        let (layouts, node) =
            Self::with_state(self.id, self.ui.ctx().clone(), |state: &mut TaffyState| {
                if state.taffy.dirty(state.node).unwrap()
                    || self.ui.available_size() != state.last_size
                {
                    state.last_size = self.ui.available_size();
                    println!("dirty");

                    let mut content_fns = unsafe {
                        mem::transmute::<&mut Vec<ContentFn<'a>>, &mut Vec<ContentFn<'static>>>(
                            &mut self.content_fns,
                        )
                    };

                    let result = state
                        .taffy
                        .compute_layout_with_context(
                            state.node,
                            Size {
                                width: AvailableSpace::Definite(self.ui.available_width()),
                                height: AvailableSpace::Definite(self.ui.available_height()),
                            },
                            content_fns,
                        )
                        .unwrap();
                }

                let layouts: Vec<_> = state
                    .children
                    .iter()
                    .map(|(id, child, egui_layout)| {
                        let layout = state.taffy.layout(child.clone()).unwrap();

                        (*layout, *egui_layout)
                    })
                    .collect();

                let node = state.taffy.layout(state.node).unwrap();

                (layouts, *node)
            });

        layouts
            .iter()
            .zip(self.content_fns)
            .for_each(|((layout, egui_layout), mut content)| {
                let offset = self.ui.next_widget_position().to_vec2();

                let rect = egui::Rect::from_min_size(
                    Pos2::new(layout.location.x, layout.location.y) + offset,
                    egui::Vec2::new(layout.size.width, layout.size.height),
                );

                if !self.ui.is_rect_visible(rect) {
                    return;
                }

                let mut child = self.ui.child_ui(rect, *egui_layout);

                content(&mut child);
            });

        self.ui
            .allocate_space(egui::Vec2::new(node.size.width, node.size.height));
    }
}
