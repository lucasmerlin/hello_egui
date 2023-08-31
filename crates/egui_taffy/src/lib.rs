use std::collections::HashMap;
use std::mem;

use egui::util::IdTypeMap;
use egui::{Context, Id, LayerId, Order, Pos2, Ui};
use taffy::prelude::*;

type Node = NodeId;

struct TaffyState<'f> {
    taffy: Taffy<MeasureFunc<Vec<Option<ContentFn<'f>>>>>,

    children: Vec<EguiTaffyNode>,

    root_node: Node,
    node_style: Style,

    last_size: egui::Vec2,
}

impl<'f> Clone for TaffyState<'f> {
    fn clone(&self) -> Self {
        panic!("TaffyState is not cloneable")
    }
}

unsafe impl<'f> Send for TaffyState<'f> {}
unsafe impl<'f> Sync for TaffyState<'f> {}

impl<'f> TaffyState<'f> {
    pub fn new() -> Self {
        let mut taffy = Taffy::new();

        Self {
            root_node: taffy.new_with_children(Style::default(), &[]).unwrap(),
            node_style: Style::default(),
            taffy,
            children: Vec::new(),
            last_size: egui::Vec2::ZERO,
        }
    }

    pub fn begin_pass(&mut self, style: Style) {
        if self.node_style != style {
            self.node_style = style.clone();
            self.taffy.set_style(self.root_node, style).unwrap();
        }
    }
}

type ContentFn<'a> = Box<dyn FnMut(&mut Ui) + Send + 'a>;

enum EguiTaffyNode {
    Leaf(Id, Node, egui::Layout, Node),
    Node(Node, Node),
}

pub struct TaffyPass<'a, 'f> {
    id: Id,

    ui: &'a mut Ui,

    content_fns: Vec<Option<ContentFn<'f>>>,

    current_node: Node,
    current_node_index: usize,

    measure_ctx: Context,
}

impl<'a, 'f> TaffyPass<'a, 'f> {
    fn with_state<T>(id: Id, ctx: Context, f: impl FnOnce(&mut TaffyState) -> T) -> T {
        ctx.data_mut(|data: &mut IdTypeMap| {
            let data = data.get_temp_mut_or_insert_with(id, TaffyState::new);

            f(data)
        })
    }

    pub fn new(ui: &'a mut Ui, id: Id, style: Style) -> Self {
        let current_node = Self::with_state(id, ui.ctx().clone(), |state| {
            state.begin_pass(style);
            state.root_node
        });

        let measure_ctx = Context::default();
        let _ = measure_ctx.run(Default::default(), |_| {});

        Self {
            id,
            ui,
            content_fns: vec![],
            measure_ctx,
            current_node,
            current_node_index: 0,
        }
    }

    pub fn add_children_with_ui(
        &mut self,
        style: Style,
        content: impl FnMut(&mut Ui) + Send + 'f,
        f: impl FnMut(&mut TaffyPass<'a, 'f>),
    ) {
        self._add_children(style, Some(Box::new(content)), f);
    }

    pub fn add_children(&mut self, style: Style, f: impl FnMut(&mut TaffyPass<'a, 'f>)) {
        self._add_children(style, None, f);
    }

    fn _add_children(
        &mut self,
        style: Style,
        content: Option<ContentFn<'f>>,
        mut f: impl FnMut(&mut TaffyPass<'a, 'f>),
    ) {
        let previous_node = self.current_node;
        let previous_node_index = self.current_node_index;

        Self::with_state(self.id, self.ui.ctx().clone(), |state| {
            let index = self.content_fns.len();
            self.content_fns.push(content);

            if let Some(c_node) = state.children.get_mut(index) {
                if let EguiTaffyNode::Node(node, _parent) = c_node {
                    if state.taffy.style(*node).unwrap() != &style {
                        state.taffy.set_style(*node, style).unwrap();
                    }
                    self.current_node = *node;
                    self.current_node_index = 0;
                } else {
                    let node = state.taffy.new_leaf(style).unwrap();
                    state
                        .taffy
                        .replace_child_at_index(previous_node, previous_node_index, node)
                        .unwrap();
                    *c_node = EguiTaffyNode::Node(node, previous_node);
                    self.current_node = node;
                    self.current_node_index = 0;
                }
            } else {
                let node = state.taffy.new_leaf(style).unwrap();
                state.taffy.add_child(previous_node, node).unwrap();
                state
                    .children
                    .push(EguiTaffyNode::Node(node, previous_node));
                self.current_node = node;
                self.current_node_index = 0;
            }
        });

        f(self);

        self.current_node = previous_node;
        self.current_node_index = previous_node_index + 1;
    }

    pub fn add(
        &mut self,
        id: Id,
        style: Style,
        layout: egui::Layout,
        content: impl FnMut(&mut Ui) + Send + 'f,
    ) {
        Self::with_state(self.id, self.ui.ctx().clone(), |state| {
            let content_idx = self.content_fns.len();
            self.content_fns.push(Some(Box::new(content)));

            let node_idx = self.current_node_index;
            self.current_node_index += 1;

            if let Some(EguiTaffyNode::Leaf(c_id, c_node, c_layout, c_parent)) =
                state.children.get_mut(content_idx)
            {
                if *c_id != id
                    || state.taffy.style(*c_node).unwrap() != &style
                    || *c_layout != layout
                    || *c_parent != self.current_node
                {
                    *c_id = id;
                    *c_layout = layout;
                    *c_parent = self.current_node;
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
                                let f = data.get_mut(content_idx).unwrap();

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
                                        main_wrap: layout.main_wrap,
                                        ..egui::Layout::default()
                                    },
                                    |ui| {
                                        f.as_mut().expect("Expected content fn to be set!")(ui);
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

                let egui_node = EguiTaffyNode::Leaf(id, node, layout, self.current_node);

                if content_idx >= state.children.len() {
                    state.children.push(egui_node);
                    state.taffy.add_child(self.current_node, node).unwrap();
                } else {
                    state.children[content_idx] = egui_node;
                    state
                        .taffy
                        .replace_child_at_index(self.current_node, node_idx, node)
                        .unwrap();
                }

                // state
                //     .taffy
                //     .set_children(
                //         state.root_node,
                //         &state
                //             .children
                //             .iter()
                //             .map(|(id, node, layout)| node.clone())
                //             .collect::<Vec<_>>(),
                //     )
                //     .unwrap();
            }
        });
    }

    pub fn show(mut self) {
        let (layouts, node) =
            //Self::with_state(self.id, self.ui.ctx().clone(), |state: &mut TaffyState| {
            self.ui.ctx().data_mut(|data: &mut IdTypeMap| {
                let mut state: &mut TaffyState = data.get_temp_mut_or_insert_with(self.id, TaffyState::new);

                if state.taffy.dirty(state.root_node).unwrap()
                    || self.ui.available_size() != state.last_size
                {
                    state.last_size = self.ui.available_size();

                    // let mut content_fns = unsafe {
                    //     mem::transmute::<
                    //         &mut Vec<Option<ContentFn<'a>>>,
                    //         &mut Vec<Option<ContentFn<'static>>>,
                    //     >(&mut self.content_fns)
                    // };

                    let state = unsafe {
                        mem::transmute::<
                            &mut TaffyState<'static>,
                            &mut TaffyState<'f>,
                        >(&mut state)
                    };

                    state
                        .taffy
                        .compute_layout_with_context(
                            state.root_node,
                            Size {
                                width: AvailableSpace::Definite(self.ui.available_width()),
                                height: AvailableSpace::Definite(self.ui.available_height()),
                            },
                            &mut self.content_fns,
                        )
                        .unwrap();
                }

                let mut parent_layouts = HashMap::new();

                let root_layout = state.taffy.layout(state.root_node).unwrap();

                let rect = egui::Rect::from_min_size(
                    Pos2::new(root_layout.location.x, root_layout.location.y),
                    egui::Vec2::new(root_layout.size.width, root_layout.size.height),
                );

                parent_layouts.insert(Into::<u64>::into(state.root_node), rect);

                let layouts: Vec<_> = state
                    .children
                    .iter()
                    .map(|node| match node {
                        EguiTaffyNode::Leaf(_id, child, egui_layout, parent) => {
                            let parent_rect = parent_layouts.get(&(*parent).into()).unwrap();

                            let layout = state.taffy.layout(*child).unwrap();

                            let rect = egui::Rect::from_min_size(
                                Pos2::new(layout.location.x, layout.location.y),
                                egui::Vec2::new(layout.size.width, layout.size.height),
                            );

                            let rect = rect.translate(parent_rect.min.to_vec2());

                            (rect, *egui_layout)
                        }
                        EguiTaffyNode::Node(node, parent) => {
                            let parent_rect = parent_layouts.get(&(*parent).into()).unwrap();

                            let layout = state.taffy.layout(*node).unwrap();

                            let rect = egui::Rect::from_min_size(
                                Pos2::new(layout.location.x, layout.location.y),
                                egui::Vec2::new(layout.size.width, layout.size.height),
                            );

                            let rect = rect.translate(parent_rect.min.to_vec2());

                            parent_layouts.insert(Into::<u64>::into(*node), rect);

                            (rect, egui::Layout::default())
                        }
                    })
                    .collect();

                let node = state.taffy.layout(state.root_node).unwrap();

                (layouts, *node)
            });

        layouts
            .iter()
            .zip(self.content_fns)
            .for_each(|((rect, egui_layout), content)| {
                if let Some(mut content) = content {
                    let offset = self.ui.next_widget_position().to_vec2();

                    let rect = rect.translate(offset);

                    if !self.ui.is_rect_visible(rect) {
                        return;
                    }

                    let mut child = self.ui.child_ui(rect, *egui_layout);

                    content(&mut child);
                }
            });

        self.ui
            .allocate_space(egui::Vec2::new(node.size.width, node.size.height));
    }
}
