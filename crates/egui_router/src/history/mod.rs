mod browser;

pub use browser::BrowserHistory;

pub trait History {
    fn update(&mut self, ctx: &egui::Context) -> impl Iterator<Item = HistoryEvent> + 'static;
    fn active_route(&self) -> Option<String>;
    fn push(&mut self, url: &str, state: u32) -> HistoryResult;
    fn replace(&mut self, url: &str, state: u32) -> HistoryResult;
    fn back(&mut self) -> HistoryResult;
    fn forward(&mut self) -> HistoryResult;
}

#[derive(Debug, Clone)]
pub(crate) struct HistoryEvent {
    pub(crate) location: String,
    pub(crate) state: Option<u32>,
}

type HistoryResult<T = ()> = Result<T, HistoryError>;

#[derive(Debug)]
pub enum HistoryError {
    JsError(wasm_bindgen::JsValue),
}

impl From<wasm_bindgen::JsValue> for HistoryError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        Self::JsError(value)
    }
}
