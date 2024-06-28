#[cfg(target_arch = "wasm32")]
mod browser;
mod memory;

use crate::history;
#[cfg(target_arch = "wasm32")]
pub use browser::BrowserHistory;
pub use memory::MemoryHistory;

pub trait History {
    fn update(&mut self, ctx: &egui::Context) -> impl Iterator<Item = HistoryEvent> + 'static;
    fn active_route(&self) -> Option<(String, Option<u32>)>;
    fn push(&mut self, url: &str, state: u32) -> HistoryResult;
    fn replace(&mut self, url: &str, state: u32) -> HistoryResult;
    fn back(&mut self) -> HistoryResult;
    fn forward(&mut self) -> HistoryResult;
}

#[cfg(target_arch = "wasm32")]
pub type DefaultHistory = history::BrowserHistory;
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultHistory = history::MemoryHistory;

#[derive(Debug, Clone)]
pub(crate) struct HistoryEvent {
    pub(crate) location: String,
    pub(crate) state: Option<u32>,
}

type HistoryResult<T = ()> = Result<T, HistoryError>;

#[derive(Debug)]
pub enum HistoryError {
    #[cfg(target_arch = "wasm32")]
    JsError(wasm_bindgen::JsValue),
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for HistoryError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::JsError(value)
    }
}
