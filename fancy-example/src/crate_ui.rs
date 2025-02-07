use casey::snake;
use egui::{Button, ScrollArea, Ui, Widget};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use Crate::{
    EguiAnimation, EguiDnd, EguiFlex, EguiForm, EguiInbox, EguiInfiniteScroll, EguiPullToRefresh,
    EguiRouter, EguiSuspense, EguiThumbhash, EguiVirtualList,
};

use crate::shared_state::SharedState;
use crate::{demo_area, FancyMessage};

pub const README_CONTENT_SEPARATOR: &str = "[content]:<>";

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Crate {
    EguiAnimation,
    EguiDnd,
    EguiFlex,
    EguiForm,
    EguiInbox,
    EguiInfiniteScroll,
    EguiPullToRefresh,
    EguiRouter,
    EguiSuspense,
    EguiThumbhash,
    EguiVirtualList,
}

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

            pub fn docs_link(&self) -> &'static str {
                match self {
                    $(Self::$name => concat!("https://docs.rs/", snake!(stringify!($name))),)*
                }
            }

            pub fn crates_io_link(&self) -> &'static str {
                match self {
                    $(Self::$name => concat!("https://crates.io/crates/", snake!(stringify!($name))),)*
                }
            }

            pub fn github_link(&self) -> &'static str {
                match self {
                    $(Self::$name => concat!("https://github.com/lucasmerlin/hello_egui/tree/main/crates/", snake!(stringify!($name))),)*
                }
            }
        }
    };
}

crate_impl!(
    EguiAnimation,
    EguiDnd,
    EguiFlex,
    EguiForm,
    EguiInbox,
    EguiInfiniteScroll,
    EguiPullToRefresh,
    EguiRouter,
    EguiSuspense,
    EguiThumbhash,
    EguiVirtualList
);

impl Crate {
    pub fn short_name(self) -> &'static str {
        if let Some(name) = self.name().strip_prefix("egui_") {
            name
        } else {
            self.name()
        }
    }

    pub fn short_description(self) -> &'static str {
        match self {
            EguiAnimation => "Animation utilities for egui",
            EguiDnd => "Drag and drop sorting for egui",
            EguiFlex => "Flex layout for egui",
            EguiForm => "Form validation for egui",
            EguiInbox => "Channel with ergonomics optimized for egui",
            EguiInfiniteScroll => "Infinite scroll widget for egui",
            EguiPullToRefresh => "Pull to refresh widget for egui",
            EguiRouter => "SPA-like router for egui",
            EguiSuspense => "Suspense widget for egui for ergonomic data fetching",
            EguiThumbhash => "Image loading and caching for egui",
            EguiVirtualList => {
                "Virtual list widget for egui with support for varying heights and custom layouts"
            }
        }
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
            let response = Button::new(crate_used.name()).corner_radius(16.0).ui(ui);

            let response = ui.interact(response.rect, response.id, egui::Sense::click());

            if response.clicked() {
                shared_state
                    .tx
                    .send(FancyMessage::Navigate(format!(
                        "/crate/{}",
                        crate_used.name()
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

// pub fn crate_button_ui(ui: &mut Ui, name: &str, selected: bool) -> Response {
//     ui.scope(|ui| {
//         ui.spacing_mut().button_padding = egui::vec2(6.0, 4.0);
//         Button::new(name).selected(selected).rounding(16.0).ui(ui)
//     })
//     .inner
// }

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

    pub fn ui(&mut self, ui: &mut Ui, item: Crate) {
        demo_area(ui, item.name(), 1000.0, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.hyperlink_to("docs.rs", item.docs_link());
                    ui.hyperlink_to("crates.io", item.crates_io_link());
                    ui.hyperlink_to("github", item.github_link());
                });

                let readme = item.readme();

                let readme_split = readme
                    .split_once(README_CONTENT_SEPARATOR)
                    .map_or(readme, |(_, a)| a);

                // TODO: Find a better solution or cache string
                let readme_split = readme_split.replace(" no_run", "");

                CommonMarkViewer::new().show(ui, &mut self.markdown_cache, &readme_split);
            });
        });
    }
}
