#[cfg(target_arch = "wasm32")]
mod browser;
mod memory;

use crate::history;
#[cfg(target_arch = "wasm32")]
pub use browser::BrowserHistory;
pub use memory::MemoryHistory;

/// Implement this trait to provide a custom history implementation
pub trait History {
    /// Check whether there is a new `HistoryEvent` (a navigation occurred)
    fn update(&mut self, ctx: &egui::Context) -> impl Iterator<Item = HistoryEvent> + 'static;
    /// Get the currently active route
    fn active_route(&self) -> Option<(String, Option<u32>)>;
    /// Push a new route to the history
    fn push(&mut self, url: &str, state: u32) -> HistoryResult;
    /// Replace the current route in the history
    fn replace(&mut self, url: &str, state: u32) -> HistoryResult;
    /// Go back in the history
    fn back(&mut self) -> HistoryResult;
    /// Go forward in the history
    fn forward(&mut self) -> HistoryResult;
}

/// Default history. Uses [BrowserHistory] on wasm32 and [MemoryHistory] otherwise
#[cfg(target_arch = "wasm32")]
pub type DefaultHistory = history::BrowserHistory;
/// Default history. Uses [`BrowserHistory`] on wasm32 and [`MemoryHistory`] otherwise
#[cfg(not(target_arch = "wasm32"))]
pub type DefaultHistory = history::MemoryHistory;

/// Result type returned by [`History::update`]
#[derive(Debug, Clone)]
pub struct HistoryEvent {
    /// The path we are navigating to
    pub location: String,
    /// The state of the history
    pub state: Option<u32>,
}

/// History Result type
type HistoryResult<T = ()> = Result<T, HistoryError>;

/// History error
#[derive(Debug, thiserror::Error)]
pub enum HistoryError {
    /// Updating the browser history failed
    #[cfg(target_arch = "wasm32")]
    #[error("History error: {0:?}")]
    JsError(wasm_bindgen::JsValue),
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for HistoryError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::JsError(value)
    }
}
