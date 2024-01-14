use casey::snake;
use egui::{Button, OpenUrl, Response, ScrollArea, Ui, Widget};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::shared_state::SharedState;
use crate::sidebar::ActiveElement;
use crate::{demo_area, FancyMessage};
use Crate::*;

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Crate {
    EguiAnimation,
    EguiDnd,
    EguiInbox,
    EguiInfiniteScroll,
    EguiPullToRefresh,
    EguiSuspense,
    EguiThumbhash,
    EguiVirtualList,
}

// Gets passed a list of crates and generates the following functions:
/*
pub fn name(&self) -> &'static str {
    match self {
        Self::Animation => "egui_animation",
        Self::Dnd => "egui_dnd",
        Self::Inbox => "egui_inbox",
        Self::InfiniteScroll => "egui_infinite_scroll",
        Self::PullToRefresh => "egui_pull_to_refresh",
        Self::Suspense => "egui_suspense",
        Self::Thumbhash => "egui_thumbhash",
        Self::VirtualList => "egui_virtual_list",
    }
}*/
macro_rules! crate_impl {
    ($($name:ident),*) => {
        pub const ALL_CRATES: &[Crate] = &[$($name),*];

        impl Crate {
            pub fn name(&self) -> &'static str {
                match self {
                    $(Self::$name => snake!(stringify!($name)),)*
                }
            }

            pub fn readme(&self) -> &'static str {
                match self {
                    $(Self::$name => include_str!(concat!("../../crates/", snake!(stringify!($name)), "/README.md")),)*
                }
            }
        }
    };
}

crate_impl!(
    EguiAnimation,
    EguiDnd,
    EguiInbox,
    EguiInfiniteScroll,
    EguiPullToRefresh,
    EguiSuspense,
    EguiThumbhash,
    EguiVirtualList
);

impl Crate {
    pub fn short_name(&self) -> &'static str {
        if let Some(name) = self.name().strip_prefix("egui_") {
            name
        } else {
            self.name()
        }
    }

    pub fn short_description(&self) -> &'static str {
        match self {
            Self::EguiAnimation => "Animation utilities for egui",
            Self::EguiDnd => "Drag and drop sorting for egui",
            Self::EguiInbox => "Channel with ergonomics optimized for egui",
            Self::EguiInfiniteScroll => "Infinite scroll widget for egui",
            Self::EguiVirtualList => {
                "Virtual list widget for egui with support for varying heights and custom layouts"
            }
            Self::EguiPullToRefresh => "Pull to refresh widget for egui",
            Self::EguiSuspense => "Suspense widget for egui for ergonomic data fetching",
            Self::EguiThumbhash => "Image loading and caching for egui",
        }
    }

    pub fn crates_io_link(&self) -> String {
        format!("https://crates.io/crates/{}", self.name())
    }
}

pub struct CrateUsage {
    crate_used: Crate,
    used_for: Option<&'static str>,
}

impl CrateUsage {
    pub const fn new(crate_used: Crate, used_for: &'static str) -> Self {
        Self {
            crate_used,
            used_for: Some(used_for),
        }
    }

    pub const fn simple(crate_used: Crate) -> Self {
        Self {
            crate_used,
            used_for: None,
        }
    }
}

pub fn crate_usage_ui(ui: &mut Ui, crates: &[CrateUsage], shared_state: &SharedState) {
    ui.separator();

    ui.small("Crates used:");
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.spacing_mut().item_spacing.y = 4.0;

        ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);

        for CrateUsage {
            crate_used,
            used_for,
        } in crates
        {
            let response = Button::new(crate_used.name()).rounding(16.0).ui(ui);

            let response = ui.interact(response.rect, response.id, egui::Sense::click());

            if response.clicked() {
                shared_state
                    .tx
                    .send(FancyMessage::SelectPage(ActiveElement::select_crate(
                        crate_used,
                    )))
                    .ok();
            }

            response.on_hover_ui(|ui| {
                ui.label(crate_used.short_description());

                if let Some(used_for) = used_for {
                    ui.label("Used for:");
                    ui.label(*used_for);
                }
            });
        }
    });
}

pub fn crate_button_ui(ui: &mut Ui, name: &str, selected: bool) -> Response {
    ui.scope(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);
        Button::new(name).selected(selected).rounding(16.0).ui(ui)
    })
    .inner
}

#[macro_export]
macro_rules! crate_usage {
    ($($crate_usage:tt)*) => {
        fn crates(&self) -> &'static [CrateUsage] {
            const CRATES: &[CrateUsage] = &[$($crate_usage)*];
            CRATES
        }
    };
}

pub struct CrateUi {
    markdown_cache: CommonMarkCache,
}

impl CrateUi {
    pub fn new() -> Self {
        Self {
            markdown_cache: CommonMarkCache::default(),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, item: &Crate) {
        demo_area(ui, item.name(), 1000.0, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                let readme = item.readme();
                CommonMarkViewer::new(item.name()).show(
                    ui,
                    &mut self.markdown_cache,
                    readme
                        .strip_prefix(&format!("# {}", item.name()))
                        .unwrap_or(readme),
                );
            });
        });
    }
}
