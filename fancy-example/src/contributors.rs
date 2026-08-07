use egui::{Frame, Id, Image, ScrollArea, Ui, Vec2};
use serde::Deserialize;
use std::fmt::Debug;
use std::hash::Hash;

use crate::crate_ui::{crate_usage_ui, Crate, CrateUsage};
use crate::demo_area;
use crate::example::{Example, ExampleTrait};
use crate::shared_state::SharedState;
use egui_dnd::{dnd, DragDropConfig};
use egui_infinite_scroll::{InfiniteScroll, LoadingState};

/// How many contributors we request per page. Small enough that the list actually paginates,
/// which is the point of the example.
const PER_PAGE: usize = 30;

pub const CONTRIBUTORS_EXAMPLE: Example = Example {
    name: "Contributors",
    slug: "contributors",
    crates: &[
        CrateUsage::simple(Crate::EguiDnd),
        CrateUsage::simple(Crate::EguiInfiniteScroll),
    ],
    get: || Box::new(Contributors::new()),
};

#[derive(Deserialize, Debug)]
pub struct Contributor {
    pub login: String,
    pub html_url: String,
    pub avatar_url: String,
}

#[cfg(feature = "mock")]
fn example_contributors() -> Vec<Contributor> {
    let dir = env!("CARGO_MANIFEST_DIR");
    vec![Contributor {
        login: "lucasmerlin".to_string(),
        html_url: "https://github.com/lucasmerlin".to_string(),
        avatar_url: format!("file://{dir}/src/egui.png"),
    }]
}

impl Hash for Contributor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.login.hash(state);
    }
}

/// Returns the cursor for the page after `page`, or `None` if `contributors` was the last page.
///
/// GitHub sends at most [`PER_PAGE`] contributors per page, so a page that isn't full means we've
/// seen the last contributor. Returning `None` makes [`InfiniteScroll`] stop asking for more, which
/// matters a lot here: it keeps calling the loader as long as it hands back a cursor, so always
/// returning one makes it request an empty page every single frame once the list is exhausted.
fn next_page(page: usize, contributors: &[Contributor]) -> Option<usize> {
    (contributors.len() == PER_PAGE).then_some(page + 1)
}

/// Turns a response from the GitHub contributors api into items and the cursor for the next page.
#[cfg(not(feature = "mock"))]
fn parse_contributors(
    page: usize,
    result: ehttp::Result<ehttp::Response>,
) -> Result<(Vec<Contributor>, Option<usize>), String> {
    let response = result.map_err(|err| format!("Failed to fetch contributors: {err}"))?;

    // ehttp reports http errors as a successful request, so we have to check the status ourselves.
    // Unauthenticated requests are rate limited to 60 per hour, which is the error you're most
    // likely to run into here.
    if !response.ok {
        return Err(format!(
            "Failed to fetch contributors: {} {}",
            response.status, response.status_text
        ));
    }

    let contributors = serde_json::from_slice::<Vec<Contributor>>(&response.bytes)
        .map_err(|err| format!("Failed to parse contributors: {err}"))?;

    let next_page = next_page(page, &contributors);
    Ok((contributors, next_page))
}

pub struct Contributors {
    infinite_scroll: InfiniteScroll<Contributor, usize>,
}

impl ExampleTrait for Contributors {
    fn ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        self.contributors_ui(ui, shared_state);
    }
}

impl Default for Contributors {
    fn default() -> Self {
        Self::new()
    }
}

impl Contributors {
    pub fn new() -> Self {
        let mut infinite_scroll = InfiniteScroll::new();
        infinite_scroll.virtual_list.hide_on_resize(None);

        Self {
            infinite_scroll: infinite_scroll.end_loader(|cursor, callback| {
                let page = cursor.unwrap_or(1);

                #[cfg(feature = "mock")]
                {
                    let contributors = example_contributors();
                    let next_page = next_page(page, &contributors);
                    callback(Ok((contributors, next_page)));
                }

                #[cfg(not(feature = "mock"))]
                ehttp::fetch(
                    ehttp::Request::get(format!(
                        "https://api.github.com/repos/lucasmerlin\
                            /hello_egui/contributors?per_page={PER_PAGE}&page={page}"
                    )),
                    move |result| {
                        callback(parse_contributors(page, result));
                    },
                );
            }),
        }
    }

    pub fn contributors_ui(&mut self, ui: &mut Ui, shared_state: &mut SharedState) {
        demo_area(ui, CONTRIBUTORS_EXAMPLE.name, 300.0, |ui| {
            ScrollArea::vertical()
                .max_height(250.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("Contribute to");
                        ui.hyperlink_to(
                            " egui_dnd on GitHub ",
                            "https://github.com/lucasmerlin/hello_egui/tree/main/crates/egui_dnd",
                        );
                        ui.label("to be listed here!");
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("On mobile you can drag to scroll and hold + drag to sort items.");
                    });
                    self.contributors_dnd_ui(ui);
                    self.loading_state_ui(ui);
                });

            crate_usage_ui(ui, CONTRIBUTORS_EXAMPLE.crates, shared_state);
        });
    }

    /// Shows a spinner while more contributors are on the way, and the error (plus a retry button)
    /// if a request failed. Without this the list would just silently stop growing.
    fn loading_state_ui(&mut self, ui: &mut Ui) {
        let error = match self.infinite_scroll.bottom_loading_state() {
            LoadingState::Idle | LoadingState::Loading | LoadingState::Loaded(..) => {
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.spinner();
                });
                return;
            }
            LoadingState::Error(error) => error.clone(),
            LoadingState::NoMoreItems => return,
        };

        ui.horizontal_wrapped(|ui| {
            ui.add_space(10.0);
            ui.colored_label(ui.visuals().error_fg_color, error);
        });
        ui.horizontal(|ui| {
            ui.add_space(10.0);
            if ui.button("Retry").clicked() {
                self.infinite_scroll.retry_bottom();
            }
        });
    }

    pub fn contributors_dnd_ui(&mut self, ui: &mut Ui) {
        let response = dnd(ui, "contributors_dnd")
            .with_touch_config(Some(DragDropConfig::touch_scroll()))
            .show_custom(|ui, iter| {
                self.infinite_scroll.ui(ui, 10, |ui, index, item| {
                    iter.next(ui, Id::new(&*item.login), index, true, |ui, item_handle| {
                        item_handle.ui(ui, |ui, handle, _state| {
                            ui.horizontal(|ui| {
                                handle.ui(ui, |ui| {
                                    Frame::NONE
                                        .fill(ui.visuals().faint_bg_color)
                                        .inner_margin(8.0)
                                        .outer_margin(2.0)
                                        .corner_radius(4.0)
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());

                                            let size = Vec2::new(32.0, 32.0);

                                            let image_url = if cfg!(feature = "mock") {
                                                item.avatar_url.clone()
                                            } else {
                                                format!(
                                                    "{}&s={}",
                                                    item.avatar_url,
                                                    size.x as u32 * 2
                                                )
                                            };

                                            ui.add(Image::new(image_url).fit_to_exact_size(size));

                                            ui.hyperlink_to(
                                                item.login.as_str(),
                                                item.html_url.as_str(),
                                            );
                                        });
                                });
                            });
                        })
                    });
                });
            });
        response.update_vec(&mut self.infinite_scroll.items);
    }
}

#[cfg(test)]
mod tests {
    use super::{next_page, Contributor, PER_PAGE};

    fn contributors(count: usize) -> Vec<Contributor> {
        (0..count)
            .map(|i| Contributor {
                login: format!("user{i}"),
                html_url: String::new(),
                avatar_url: String::new(),
            })
            .collect()
    }

    #[test]
    fn full_page_asks_for_the_next_one() {
        assert_eq!(next_page(1, &contributors(PER_PAGE)), Some(2));
        assert_eq!(next_page(7, &contributors(PER_PAGE)), Some(8));
    }

    /// A partial or empty page means we've seen the last contributor. If we kept handing back a
    /// cursor here the infinite scroll would request an empty page every frame, which burns
    /// through GitHub's unauthenticated rate limit within seconds.
    #[test]
    fn partial_page_ends_pagination() {
        assert_eq!(next_page(7, &contributors(PER_PAGE - 1)), None);
        assert_eq!(next_page(7, &contributors(0)), None);
    }
}
