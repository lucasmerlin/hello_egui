use crate::history::{History, HistoryEvent, HistoryResult};
use egui_inbox::UiInbox;
use js_sys::Number;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::window;

pub struct BrowserHistory {
    inbox: UiInbox<HistoryEvent>,
    history: web_sys::History,
    closure: Closure<dyn FnMut(web_sys::PopStateEvent)>,
}

impl Default for BrowserHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserHistory {
    pub fn new() -> Self {
        let window = window().unwrap();
        let (tx, inbox) = UiInbox::channel();

        let cb = Closure::wrap(Box::new(move |event: web_sys::PopStateEvent| {
            let state = event.state().as_f64().map(|n| n as u32);
            let location = web_sys::window().unwrap().location();
            let path = format!(
                "{}{}",
                location.pathname().unwrap(),
                location.search().unwrap()
            );
            tx.send(HistoryEvent {
                location: path,
                state,
            })
            .ok();
        }) as Box<dyn FnMut(_)>);

        window
            .add_event_listener_with_callback("popstate", cb.as_ref().unchecked_ref())
            .unwrap();
        Self {
            inbox,
            history: window.history().unwrap(),
            closure: cb,
        }
    }
}

impl Drop for BrowserHistory {
    fn drop(&mut self) {
        window()
            .unwrap()
            .remove_event_listener_with_callback("popstate", self.closure.as_ref().unchecked_ref())
            .unwrap();
    }
}

impl History for BrowserHistory {
    fn update(&mut self, ctx: &egui::Context) -> impl Iterator<Item = HistoryEvent> + 'static {
        self.inbox.read(ctx)
    }

    fn active_route(&self) -> Option<String> {
        let location = window().unwrap().location();
        let path = format!(
            "{}{}",
            location.pathname().unwrap(),
            location.search().unwrap()
        );
        Some(path)
    }

    fn push(&mut self, url: &str, state: u32) -> HistoryResult {
        self.history
            .push_state_with_url(&Number::from(state), "", Some(url))?;
        Ok(())
    }

    fn replace(&mut self, url: &str, state: u32) -> HistoryResult {
        self.history
            .replace_state_with_url(&Number::from(state), "", Some(url))?;
        Ok(())
    }

    fn back(&mut self) -> HistoryResult {
        self.history.back()?;
        Ok(())
    }

    fn forward(&mut self) -> HistoryResult {
        self.history.forward()?;
        Ok(())
    }
}
