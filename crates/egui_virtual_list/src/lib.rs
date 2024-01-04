use egui::{Pos2, Rect, Ui, Vec2};
use std::ops::Range;

pub struct VirtualListResponse {
    /// The range of items that was displayed
    pub item_range: Range<usize>,

    /// Any items in this range are now visible
    pub newly_visible_items: Range<usize>,
    /// Any items in this range are no longer visible
    pub hidden_items: Range<usize>,
}

#[derive(Debug)]
struct RowData {
    range: Range<usize>,
    pos: Pos2,
}

#[derive(Debug)]
pub struct VirtualList {
    rows: Vec<RowData>,

    previous_item_range: Range<usize>,

    // The index of the first item that has an unknown rect
    last_known_row_index: Option<usize>,

    average_row_size: Option<Vec2>,
    average_items_per_row: Option<f32>,

    // We will recalculate every item's rect if the scroll area's width changes
    last_width: f32,

    max_rows_calculated_per_frame: usize,

    over_scan: f32,

    // If set, the list will scroll by this many items from the top.
    // Useful when items at the top are added, and the scroll position should be maintained.
    // The value should be the number of items that were added at the top.
    items_inserted_at_start: Option<usize>,
}

impl Default for VirtualList {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualList {
    /// Create a new VirtualList
    ///
    /// `over_scan` is useful e.g. when used in combination with egui_dnd.
    /// Renders items this much before and after the visible area.
    pub fn new() -> Self {
        Self {
            previous_item_range: usize::MAX..usize::MAX,
            last_known_row_index: None,
            last_width: 0.0,
            average_row_size: None,
            rows: vec![],
            average_items_per_row: None,
            max_rows_calculated_per_frame: 1000,
            over_scan: 0.0,
            items_inserted_at_start: None,
        }
    }

    /// Set the number of items that were added at the top.
    pub fn items_inserted_at_start(&mut self, scroll_top_items: usize) {
        self.items_inserted_at_start = Some(scroll_top_items);
    }

    /// Layout gets called with the index of the first item that should be displayed.
    /// It should return the number of items that were displayed.
    pub fn ui_custom_layout(
        &mut self,
        ui: &mut Ui,
        length: usize,
        mut layout: impl FnMut(&mut Ui, usize) -> usize,
    ) -> VirtualListResponse {
        if ui.available_width() != self.last_width {
            self.last_known_row_index = None;
            self.last_width = ui.available_width();
            self.rows.clear();
        }

        // Start of the scroll area (basically scroll_offset + whatever is above the scroll area)
        let min = ui.next_widget_position().to_vec2();

        let mut row_start_index = self.last_known_row_index.unwrap_or(0);

        // This calculates the visible rect inside the scroll area
        // Should be equivalent to to viewport from ScrollArea::show_viewport(), offset by whatever is above the scroll area
        let visible_rect = ui.clip_rect().translate(-min);

        let visible_rect = visible_rect.expand2(Vec2::new(0.0, self.over_scan));

        let mut index_offset = 0;

        // Calculate the added_height for items that were added at the top and scroll by that amount
        // to maintain the scroll position
        let scroll_items_top_step_2 =
            if let Some(scroll_top_items) = self.items_inserted_at_start.take() {
                let mut measure_ui = ui.child_ui(ui.max_rect(), *ui.layout());
                measure_ui.set_visible(false);

                let start_height = measure_ui.next_widget_position();
                for i in 0..scroll_top_items {
                    layout(&mut measure_ui, i);
                }
                let end_height = measure_ui.next_widget_position();

                let added_height = end_height.y - start_height.y + ui.spacing().item_spacing.y;

                // TODO: Ideally we should be able to use scroll_with_delta here but that doesn't work
                // until https://github.com/emilk/egui/issues/2783 is fixed. Before, scroll_to_rect
                // only works when the mouse is over the scroll area.
                // ui.scroll_with_delta(Vec2::new(0.0, -added_height));
                ui.scroll_to_rect(ui.clip_rect().translate(Vec2::new(0.0, added_height)), None);

                index_offset = scroll_top_items;

                ui.ctx().request_repaint();

                Some(added_height)
            } else {
                None
            };

        // Find the first row that is visible
        loop {
            if row_start_index == 0 {
                break;
            }

            if let Some(row) = self.rows.get(row_start_index) {
                if row.pos.y <= visible_rect.min.y {
                    ui.add_space(row.pos.y);
                    break;
                }
            }
            row_start_index -= 1;
        }
        let mut current_row = row_start_index;

        let item_start_index = self
            .rows
            .get(row_start_index)
            .map(|row| row.range.start)
            .unwrap_or(0)
            + index_offset;

        let mut current_item_index = item_start_index;

        let mut iterations = 0;

        let mut first_visible_item_index = None;

        loop {
            // Bail out if we're recalculating too many items
            if iterations > self.max_rows_calculated_per_frame {
                ui.ctx().request_repaint();
                break;
            }
            iterations += 1;

            // let item = self.items.get_mut(current_row);
            if current_item_index < length {
                let pos = ui.next_widget_position() - min;
                let count = layout(ui, current_item_index);
                let size = ui.next_widget_position() - min - pos;
                let rect = Rect::from_min_size(pos, size);

                let range = current_item_index..current_item_index + count;

                if first_visible_item_index.is_none() && pos.y >= visible_rect.min.y {
                    first_visible_item_index = Some(current_item_index);
                }

                if let Some(row) = self.rows.get_mut(current_row) {
                    row.range = range;
                    row.pos = pos;
                } else {
                    self.rows.push(RowData {
                        range: current_item_index..current_item_index + count,
                        pos,
                    });

                    let size_with_space = size;

                    self.average_row_size = Some(
                        self.average_row_size
                            .map(|size| {
                                (current_row as f32 * size + size_with_space)
                                    / (current_row as f32 + 1.0)
                            })
                            .unwrap_or(size),
                    );

                    self.average_items_per_row = Some(
                        self.average_items_per_row
                            .map(|avg_count| {
                                (current_row as f32 * avg_count + count as f32)
                                    / (current_row as f32 + 1.0)
                            })
                            .unwrap_or(count as f32),
                    );

                    self.last_known_row_index = Some(current_row);
                }

                current_item_index += count;

                if rect.max.y > visible_rect.max.y {
                    break;
                }
            } else {
                break;
            }

            current_row += 1;
        }

        let item_range = first_visible_item_index.unwrap_or(item_start_index)..current_item_index;

        if item_range.end < length {
            ui.set_min_height(
                (length - item_range.end) as f32 / self.average_items_per_row.unwrap_or(1.0)
                    * self.average_row_size.unwrap_or(Vec2::ZERO).y,
            );
        }

        let mut hidden_range =
            self.previous_item_range.start..item_range.start.min(self.previous_item_range.end);
        if hidden_range.is_empty() {
            hidden_range =
                item_range.end.max(self.previous_item_range.start)..self.previous_item_range.end;
        }

        let mut visible_range = self.previous_item_range.end.max(item_range.start)..item_range.end;
        if visible_range.is_empty() {
            visible_range =
                self.previous_item_range.start..item_range.start.min(self.previous_item_range.end);
        }

        self.previous_item_range = item_range.clone();

        if let Some(added_height) = scroll_items_top_step_2 {
            // We need to add the height at the bottom or else we might not be able to scroll
            ui.add_space(added_height);
            self.reset();
        }

        VirtualListResponse {
            item_range,
            newly_visible_items: visible_range,
            hidden_items: hidden_range,
        }
    }

    pub fn reset(&mut self) {
        self.last_known_row_index = None;
        self.last_width = 0.0;
        self.average_row_size = None;
        self.rows.clear();
        self.average_items_per_row = None;
    }
}
