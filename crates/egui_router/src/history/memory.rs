use crate::history::{History, HistoryEvent, HistoryResult};
use egui::Context;
use std::iter;

#[derive(Debug, Clone, Default)]
pub struct MemoryHistory {}

impl History for MemoryHistory {
    fn update(&mut self, _ctx: &Context) -> impl Iterator<Item = HistoryEvent> + 'static {
        iter::empty()
    }

    fn active_route(&self) -> Option<(String, Option<u32>)> {
        None
    }

    fn push(&mut self, _url: &str, _state: u32) -> HistoryResult {
        Ok(())
    }

    fn replace(&mut self, _url: &str, _state: u32) -> HistoryResult {
        Ok(())
    }

    fn back(&mut self) -> HistoryResult {
        Ok(())
    }

    fn forward(&mut self) -> HistoryResult {
        Ok(())
    }
}
